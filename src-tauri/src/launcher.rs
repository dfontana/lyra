use std::sync::Arc;

use crate::{closer, config::Config, plugin_manager::PluginManager};
use lyra_plugin::{FuzzyMatchItem, OkAction, PluginName};
use nucleo_matcher::{
  pattern::{CaseMatching, Pattern},
  Config as NucleoConfig, Matcher,
};
use parking_lot::RwLock;
use serde_json::Value;
use tauri::ClipboardManager;
use tracing::error;

pub struct Launcher {
  pub config: Arc<Config>,
  pub plugins: PluginManager,
  matcher: RwLock<Matcher>,
}

impl Launcher {
  pub fn new(config: Arc<Config>, plugins: PluginManager) -> Self {
    let mut cfg = NucleoConfig::DEFAULT;
    cfg.ignore_case = true;
    cfg.prefer_prefix = true;
    Launcher {
      config,
      plugins,
      matcher: RwLock::new(Matcher::new(cfg)),
    }
  }
  pub async fn get_options(&self, search: &str) -> Vec<FuzzyMatchItem> {
    if search.is_empty() {
      // Special case, empty string == nothing back instead of everything
      return Vec::new();
    }

    Pattern::parse(search, CaseMatching::Ignore)
      .match_list(
        self
          .plugins
          .filter_to(&search)
          .iter()
          .flat_map(|pl| pl.options(search)),
        &mut *self.matcher.write(),
      )
      .iter()
      .take(self.config.get().result_count)
      .map(|(v, _)| v.to_owned())
      .chain(
        self
          .plugins
          .always_present(search)
          .iter()
          .flat_map(|pl| pl.static_items()),
      )
      .collect()
  }

  pub fn launch(&self, plugin: PluginName, selected: Value) -> Result<OkAction, Value> {
    self
      .plugins
      .get(&plugin)
      .map_err(|e| {
        error!("Failed to execute plugin: {}", e);
        "Failed to launch".into()
      })
      .and_then(|pl| pl.action(selected))
  }
}

#[tauri::command]
pub async fn search(
  launcher: tauri::State<'_, Launcher>,
  search: String,
) -> Result<Vec<(PluginName, Value)>, String> {
  let options = launcher.get_options(&search).await;
  Ok(
    options
      .iter()
      .map(|sk| (sk.source.clone(), sk.value.clone()))
      .collect(),
  )
}

#[tauri::command]
pub fn submit(
  launcher: tauri::State<'_, Launcher>,
  app_handle: tauri::AppHandle,
  for_plugin: PluginName,
  selected: Value,
  window: tauri::Window,
) -> Result<Value, Value> {
  match launcher.launch(for_plugin, selected) {
    Ok(OkAction {
      value,
      close_win: true,
      copy: true,
    }) => {
      app_handle
        .clipboard_manager()
        .write_text(value.to_string().trim_matches('"'))
        .unwrap();
      closer::close_win(&window);
      Ok(value)
    }
    Ok(OkAction {
      value,
      close_win: true,
      ..
    }) => {
      closer::close_win(&window);
      Ok(value)
    }
    Ok(OkAction { value, .. }) => Ok(value),
    Err(err) => Err(err),
  }
}
