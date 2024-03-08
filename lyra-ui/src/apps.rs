pub mod app_convert;
mod appcache;
mod applookup;

use crate::config::AppsConfig;
use crate::plugin::{
  AppState, FuzzyMatchItem, OkAction, Plugin, PluginV, PluginValue, Renderable, SearchBlocker,
};
use anyhow::{anyhow, Context};
use appcache::AppCache;
use applookup::AppLookup;
use egui::{Image, RichText};
use lyra_common::convert as lyra_convert;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

pub const PLUGIN_NAME: &'static str = "apps";

pub struct AppsPlugin {
  cfg: AppsConfig,
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
  pub fn init(cfg: AppsConfig, cache_dir: &PathBuf) -> Result<Self, anyhow::Error> {
    let cache = AppCache::load(cache_dir.join(format!("apps_icons.toml")))?;
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
