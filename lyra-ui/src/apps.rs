pub mod app_convert;
mod appcache;
mod applookup;

use crate::config::AppsConfig;
use crate::icon_ui::Icon;
use crate::plugin::{
  AppState, FuzzyMatchItem, OkAction, Plugin, PluginV, PluginValue, Renderable, SearchBlocker,
};
use anyhow::{anyhow, Context};
use applookup::AppLookup;
use egui::RichText;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

use self::appcache::AppsCache;

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
    ui.horizontal(|ui| {
      if let Ok(ico) = Icon::try_from((self.icon.as_str(), self.label.as_str())) {
        ico.render(ui);
      }
      ui.label(RichText::new(&self.label));
    });
  }
}
impl SearchBlocker for AppLaunch {}

impl AppsPlugin {
  pub fn init(cfg: AppsConfig, cache_dir: &PathBuf) -> Result<Self, anyhow::Error> {
    let cache = AppsCache::init(cache_dir.join(format!("apps_icons.toml")))?;
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
      .map(|_| OkAction {
        close_win: true,
        ..Default::default()
      })
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
