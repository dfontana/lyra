use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{anyhow, Context};
use applookup::AppLookup;
use config::{AppCache, AppConf};
use lyra_plugin::{Config, FuzzyMatchItem, OkAction, Plugin};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod applookup;
mod config;
mod convert;

pub const PLUGIN_NAME: &'static str = "apps";

pub struct AppsPlugin {
  cfg: Arc<AppConf>,
  apps: AppLookup,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppLaunch {
  pub label: String,
  pub icon: String,
  pub path: String,
}

impl AppsPlugin {
  pub fn init(conf_dir: &PathBuf, cache_dir: &PathBuf) -> Result<Self, anyhow::Error> {
    let cfg = Arc::new(AppConf(Config::load(
      conf_dir.join(format!("{}.toml", PLUGIN_NAME)),
    )?));
    let cache = AppCache::load(cache_dir.join(format!("app_icons.tml")))?;
    let apps = AppLookup {
      config: cfg.clone(),
      cache: Arc::new(cache),
    };
    apps.init().context("Failed to initialize app icons")?;
    Ok(AppsPlugin { cfg, apps })
  }
}

impl Plugin for AppsPlugin {
  fn get_config(&self) -> Value {
    serde_json::to_value((*self.cfg.0.get()).clone()).unwrap()
  }

  fn update_config(&self, updates: HashMap<String, Value>) -> Result<(), anyhow::Error> {
    self.cfg.update(updates)
  }

  fn action(&self, input: Value) -> Result<OkAction, anyhow::Error> {
    let data: AppLaunch = serde_json::from_value(input)?;
    open::that(data.path.clone())
      .map(|_| OkAction {
        value: Value::Null,
        close_win: true,
        copy: false,
      })
      .map_err(|err| anyhow!("Action failed for {:?}, err: {:?}", data.label, err))
  }

  fn options(&self, _: &str) -> Vec<FuzzyMatchItem> {
    self.apps.iter().map(AppLaunch::into).collect()
  }
}

impl From<AppLaunch> for FuzzyMatchItem {
  fn from(app: AppLaunch) -> FuzzyMatchItem {
    FuzzyMatchItem {
      value: serde_json::to_value(&app).unwrap(),
      against: Arc::new(app.label.clone()),
      source: PLUGIN_NAME.to_string(),
    }
  }
}
