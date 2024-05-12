use super::app_convert;
use crate::cacher::Cache;
use serde::{Deserialize, Serialize};
use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

pub struct AppsCache(Cache<AppsData>);

#[derive(Clone, Default, Deserialize, Serialize)]
struct AppsData {
  pub app_icons: HashMap<String, String>,
}

impl AppsCache {
  pub fn init(cache_file: PathBuf) -> Result<Self, anyhow::Error> {
    Cache::load(cache_file).map(|c| AppsCache(c))
  }

  pub fn get_app_icon(&self, updated: &Path) -> Result<String, anyhow::Error> {
    let key = updated.to_str().unwrap().to_string();
    // TODO Tricky deadlock, but ideally we would persist newly discovered items
    //      rather than waiting until next restart
    Ok(
      self
        .0
        .get()
        .app_icons
        .get(&key)
        .map(|v| v.to_string())
        .get_or_insert_with(|| app_convert::to_icon(updated).unwrap_or_default())
        .clone(),
    )
  }

  pub fn update_app_icons(&self, updated: Vec<PathBuf>) -> Result<(), anyhow::Error> {
    let new_app_icons = {
      let inner = self.0.get();
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
    self.0.update(|mut ad| ad.app_icons = new_app_icons);
    self.0.persist()
  }
}
