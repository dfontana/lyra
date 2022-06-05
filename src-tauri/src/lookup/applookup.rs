use crate::config::Config;
use glob::glob;
use std::path::PathBuf;

pub struct AppLookup {
  config: Config,
}

#[derive(Debug)]
pub struct App {
  pub label: String,
  pub path: PathBuf,
}

impl From<PathBuf> for App {
  fn from(p: PathBuf) -> Self {
    App {
      label: p
        .file_name()
        .expect("Glob returned non-file")
        .to_string_lossy()
        .to_string(),
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
  pub fn new(config: Config) -> Self {
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
  fn to_glob(&mut self, p: &PathBuf) -> String {
    format!("{}/*.{}", p.display(), self.extension)
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
            Ok(item) => Some(App::from(item)),
          }
        } else {
          self.next()
        }
      }
    }
  }
}
