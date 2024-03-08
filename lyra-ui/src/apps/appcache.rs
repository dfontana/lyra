use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::app_convert;

#[derive(Default)]
pub struct AppCache {
  cache_file: PathBuf,
  inner: RwLock<AppCacheInner>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct AppCacheInner {
  pub app_icons: HashMap<String, String>,
}

impl AppCache {
  pub fn load(file: PathBuf) -> Result<Self, anyhow::Error> {
    let config = if !file.exists() {
      info!(
        "File missing, generating default at {}",
        file.to_string_lossy()
      );
      let item = Self {
        cache_file: file,
        ..Self::default()
      };
      item.persist()?;
      item
    } else {
      let inner: AppCacheInner = toml::from_str(&fs::read_to_string(&file)?)?;
      Self {
        cache_file: file,
        inner: RwLock::new(inner),
      }
    };
    Ok(config)
  }

  fn persist(&self) -> Result<(), anyhow::Error> {
    fs::write(&self.cache_file, toml::to_string(&*self.inner.read())?)?;
    Ok(())
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
        .get_or_insert_with(|| app_convert::to_icon(updated).unwrap_or_default())
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
        .map(|(k, p)| (k, app_convert::to_icon(p).unwrap_or_default()))
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
}
