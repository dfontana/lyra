use std::{
  collections::HashMap,
  fs,
  ops::Deref,
  path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use crate::convert;

#[derive(Default)]
pub struct Config {
  conf_file: PathBuf,
  inner: RwLock<InnerConfig>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct InnerConfig {
  pub app_paths: Vec<PathBuf>,
  pub app_extension: String,
}

#[derive(Default)]
pub struct AppCache {
  cache_file: PathBuf,
  inner: RwLock<AppCacheInner>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct AppCacheInner {
  pub app_icons: HashMap<String, String>,
}

impl Config {
  // TODO: These first 3 methods are verbatim copied on every config and the only difference is the
  //       inner type. This smells like a serious case for a generic
  pub fn get(&self) -> impl Deref<Target = InnerConfig> + '_ {
    self.inner.read()
  }

  pub fn load(conf_file: PathBuf) -> Result<Self, anyhow::Error> {
    let config = if !conf_file.exists() {
      info!(
        "Config missing, generating default at {}",
        conf_file.to_string_lossy()
      );
      let config = Self {
        conf_file,
        ..Self::default()
      };
      config.persist()?;
      config
    } else {
      let inner: InnerConfig = toml::from_str(&fs::read_to_string(&conf_file)?)?;
      Self {
        conf_file,
        inner: RwLock::new(inner),
      }
    };
    Ok(config)
  }

  fn persist(&self) -> Result<(), anyhow::Error> {
    fs::write(&self.conf_file, toml::to_string(&*self.inner.read())?)?;
    Ok(())
  }

  pub fn update(&self, updates: HashMap<String, Value>) -> Result<(), anyhow::Error> {
    for (k, v) in updates {
      let status = match k.as_ref() {
        "app_paths" => self.update_paths(&v),
        "app_extension" => self.update_extension(&v),
        _ => Err(anyhow!("Unknown field")),
      };
      status.context(format!("Field failed operation: {}", k))?;
    }
    self.persist()
  }

  fn update_paths(&self, updated: &Value) -> Result<(), anyhow::Error> {
    let updated = serde_json::from_value::<Vec<PathBuf>>(updated.clone())?;
    self.inner.write().app_paths = updated;
    Ok(())
  }

  fn update_extension(&self, updated: &Value) -> Result<(), anyhow::Error> {
    let updated = serde_json::from_value::<String>(updated.clone())?;
    self.inner.write().app_extension = updated;
    Ok(())
  }
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
}
