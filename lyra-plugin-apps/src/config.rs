use std::{
  collections::HashMap,
  fs,
  ops::Deref,
  path::{Path, PathBuf},
};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::convert;

pub struct Config {
  data_dir: PathBuf,
  inner: RwLock<InnerConfig>,
}

#[derive(Deserialize, Serialize)]
pub struct InnerConfig {
  pub app_paths: Vec<PathBuf>,
  pub app_extension: String,
}

pub struct AppCache {
  data_dir: PathBuf,
  inner: RwLock<AppCacheInner>,
}

#[derive(Deserialize, Serialize)]
struct AppCacheInner {
  pub app_icons: HashMap<String, String>,
}

impl Config {
  pub fn load(data_dir: PathBuf) -> Self {
    // TODO: impl actually loading from config file
    Config {
      inner: RwLock::new(InnerConfig {
        app_paths: vec![],
        app_extension: "t".into(),
      }),
      data_dir,
    }
  }

  pub fn get(&self) -> impl Deref<Target = InnerConfig> + '_ {
    self.inner.read()
  }
}

impl AppCache {
  pub fn load(data_dir: PathBuf) -> Self {
    // TODO: impl actually loading from cache file
    AppCache {
      inner: RwLock::new(AppCacheInner {
        app_icons: HashMap::new(),
      }),
      data_dir,
    }
  }

  pub fn get_app_icon(&self, updated: &Path) -> Result<String, anyhow::Error> {
    let key = updated.to_str().unwrap().to_string();
    // TODO Tricky deadlock, but ideally we would persist newly discovered items
    //      rather than waiting until next restart
    Ok(
      self
        .inner
        .read()
        .app_icons
        .get(&key)
        .map(|v| v.to_string())
        .get_or_insert_with(|| convert::to_icon(updated).unwrap_or_default())
        .clone(),
    )
  }

  pub fn update_app_icons(&self, updated: Vec<PathBuf>) -> Result<(), anyhow::Error> {
    let new_app_icons = {
      let inner = self.inner.read();
      let mut new_app_icons: HashMap<String, String> = updated
        .iter()
        .map(|p| (p.to_str().unwrap().to_string(), p))
        .filter(|(k, _)| !inner.app_icons.contains_key(k))
        .map(|(k, p)| (k, convert::to_icon(p).unwrap_or_default()))
        .collect();
      if new_app_icons.is_empty() {
        return Ok(());
      }
      inner.app_icons.iter().for_each(|(k, v)| {
        new_app_icons.insert(k.clone(), v.clone());
      });
      new_app_icons
    };
    self.inner.write().app_icons = new_app_icons;
    self.persist()
  }

  fn persist(&self) -> Result<(), anyhow::Error> {
    fs::write(
      &self.data_dir.join(".cache"),
      toml::to_string(&*self.inner.read())?,
    )?;
    Ok(())
  }
}
