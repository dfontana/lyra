use glob::glob;
use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use crate::config::{AppCache, Config};

pub struct AppLookup {
  config: Arc<Config>,
  cache: Arc<AppCache>,
}

#[derive(Debug)]
pub struct App {
  pub label: String,
  pub icon: String,
  pub path: PathBuf,
}

impl App {
  fn from(p: PathBuf, suffix: &str, icon: String) -> Self {
    App {
      label: p
        .file_name()
        .expect("Glob returned non-file")
        .to_string_lossy()
        .trim_end_matches(suffix)
        .to_string(),
      icon,
      path: p,
    }
  }
}

pub struct AppLookupIter<T> {
  config: Arc<Config>,
  cache: Arc<AppCache>,
  // Extension to look for in paths
  extension: String,
  // Remaining paths to inspect during iteration
  paths_remaining: Vec<PathBuf>,
  // Current glob being iterated over
  current: Option<glob::Paths>,
  maker: Box<dyn Fn(PathBuf, &str, Arc<AppCache>) -> T>,
}

impl AppLookup {
  pub fn iter(&self) -> AppLookupIter<App> {
    let conf = self.config.get();
    AppLookupIter {
      // TODO: Do I need config on this iter or just cache?
      config: self.config.clone(),
      cache: self.cache.clone(),
      extension: conf.app_extension.clone(),
      paths_remaining: conf.app_paths.clone(),
      current: None,
      maker: Box::new(|p, suffix, cache| {
        let icon = cache.get_app_icon(&p).unwrap_or_default();
        App::from(p, suffix, icon)
      }),
    }
  }

  pub fn init(&self) -> Result<(), anyhow::Error> {
    let items = {
      let conf = self.config.get();
      let apps = AppLookupIter {
        config: self.config.clone(),
        cache: self.cache.clone(),
        extension: conf.app_extension.clone(),
        paths_remaining: conf.app_paths.clone(),
        current: None,
        maker: Box::new(|a, _, _| a),
      };
      apps.collect()
    };
    self.cache.update_app_icons(items)
  }
}

impl<T> AppLookupIter<T> {
  fn make_glob(&mut self, p: &Path) -> String {
    format!("{}/*{}", p.display(), self.extension)
  }
}

impl<T> Iterator for AppLookupIter<T> {
  type Item = T;

  fn next(&mut self) -> Option<Self::Item> {
    match self.current.take() {
      None => match self.paths_remaining.pop() {
        None => None,
        Some(next_path) => {
          self.current = Some(glob(self.make_glob(&next_path).as_str()).expect("Failed to glob"));
          self.next()
        }
      },
      Some(mut path) => {
        if let Some(next) = path.next() {
          // There's still more in this path, so restore the value of current
          self.current = Some(path);
          match next {
            // Skip path read errors
            Err(_) => self.next(),
            Ok(item) => Some((self.maker)(item, &self.extension, self.cache.clone())),
          }
        } else {
          self.next()
        }
      }
    }
  }
}
