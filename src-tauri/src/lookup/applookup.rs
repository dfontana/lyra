use crate::{config::Config, convert};
use glob::{glob};
use std::{
  path::{Path, PathBuf},
  sync::Arc,
};


pub struct AppLookup {
  config: Arc<Config>,
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

pub struct AppLookupIter {
  // Extension to look for in paths
  extension: String,
  // Remaining paths to inspect during iteration
  paths_remaining: Vec<PathBuf>,
  // Current glob being iterated over
  current: Option<glob::Paths>,
}

impl AppLookup {
  pub fn new(config: Arc<Config>) -> Self {
    AppLookup { config }
  }

  pub fn iter(&self) -> AppLookupIter {
    let conf = self.config.get();
    AppLookupIter {
      extension: conf.app_extension.clone(),
      paths_remaining: conf.app_paths.clone(),
      current: None,
    }
  }
}

impl AppLookupIter {
  fn to_glob(&mut self, p: &Path) -> String {
    format!("{}/*{}", p.display(), self.extension)
  }
}

impl Iterator for AppLookupIter {
  type Item = App;

  fn next(&mut self) -> Option<Self::Item> {
    match self.current.take() {
      None => match self.paths_remaining.pop() {
        None => None,
        Some(next_path) => {
          self.current = Some(glob(self.to_glob(&next_path).as_str()).expect("Failed to glob"));
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
            Ok(item) => {
              let icon = convert::to_icns(&item).unwrap_or_default();
              Some(App::from(item, &self.extension, icon))
            }
          }
        } else {
          self.next()
        }
      }
    }
  }
}
