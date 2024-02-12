mod applookup;
mod config;
mod convert;

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{
  AppState, Config, FuzzyMatchItem, OkAction, Plugin, PluginV, PluginValue, Renderable,
  SearchBlocker,
};
use anyhow::{anyhow, Context};
use applookup::AppLookup;
use config::{AppCache, AppConf};
use egui::{Image, RichText};
use lyra_common::convert as lyra_convert;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PLUGIN_NAME: &'static str = "apps";

pub struct AppsPlugin {
  cfg: Arc<AppConf>,
  apps: AppLookup,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppLaunch {
  pub label: String,
  pub icon: String,
  pub path: String,
}
impl PluginValue for AppLaunch {}
impl Renderable for AppLaunch {
  fn render(&self, ui: &mut egui::Ui, _state: &AppState) {
    let icon = lyra_convert::parse_image_data(&self.icon)
      .ok_or(anyhow!("Cannot render this image format"))
      .and_then(|(s, ext)| lyra_convert::decode_bytes(&s).map(|b| (b, ext)));

    ui.horizontal(|ui| {
      if let Ok((img, ext)) = icon {
        ui.add(
          Image::from_bytes(format!("bytes://{}.{}", self.label.to_string(), ext), img)
            .maintain_aspect_ratio(true)
            .shrink_to_fit(),
        );
      }
      ui.label(RichText::new(&self.label));
    });
  }
}
impl SearchBlocker for AppLaunch {}

impl AppsPlugin {
  pub fn init(conf_dir: &PathBuf, cache_dir: &PathBuf) -> Result<Self, anyhow::Error> {
    let cfg = Arc::new(AppConf(Config::load(
      conf_dir.join(format!("{}.toml", PLUGIN_NAME)),
    )?));
    let cache = AppCache::load(cache_dir.join(format!("app_icons.toml")))?;
    let apps = AppLookup {
      config: cfg.clone(),
      cache: Arc::new(cache),
    };
    apps.init().context("Failed to initialize app icons")?;
    Ok(AppsPlugin { cfg, apps })
  }
}

impl Plugin for AppsPlugin {
  type PV = AppLaunch;

  fn get_config(&self) -> Value {
    serde_json::to_value((*self.cfg.0.get()).clone()).unwrap()
  }

  fn update_config(&self, updates: HashMap<String, Value>) -> Result<(), anyhow::Error> {
    self.cfg.update(updates)
  }

  fn action(&self, input: &AppLaunch) -> Result<OkAction, anyhow::Error> {
    open::that(input.path.clone())
      .map(|_| OkAction { close_win: true })
      .map_err(|err| anyhow!("Action failed for {:?}, err: {:?}", input.label, err))
  }

  fn options(&self, _: &str) -> Vec<FuzzyMatchItem> {
    self.apps.iter().map(AppLaunch::into).collect()
  }
}

impl From<AppLaunch> for FuzzyMatchItem {
  fn from(app: AppLaunch) -> FuzzyMatchItem {
    FuzzyMatchItem {
      against: Arc::new(app.label.clone()),
      value: PluginV::Apps(app),
      source: PLUGIN_NAME.to_string(),
    }
  }
}
