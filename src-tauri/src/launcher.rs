use crate::{
  closer,
  config::Config,
  plugin_manager::PluginManager,
};
use itertools::Itertools;
use lyra_plugin::{SkimmableOption, PluginName, OkAction};
use serde_json::Value;
use skim::prelude::*;
use tauri::ClipboardManager;
use tracing::error;

pub struct Launcher {
  pub config: Arc<Config>,
  pub plugins: PluginManager,
}

impl Launcher {
  fn options_to_receiver(&self, search: &str) -> SkimItemReceiver {
    let (tx_items, rx_items): (SkimItemSender, SkimItemReceiver) = unbounded();
    self
      .plugins
      .filter_to(&search)
      .iter()
      .flat_map(|pl| pl.skim(search))
      .for_each(|sk| {
        let _ = tx_items.send(Arc::new(sk.clone()));
      });
    drop(tx_items); // indicates that all items have been sent
    rx_items
  }

  pub async fn get_options(&self, search: &str) -> Vec<SkimmableOption> {
    if search.is_empty() {
      // Special case, empty string == nothing back instead of everything
      return Vec::new();
    }
    let fuzzy_engine = ExactOrFuzzyEngineFactory::builder()
      .exact_mode(false)
      .fuzzy_algorithm(FuzzyAlgorithm::SkimV2)
      .build()
      .create_engine_with_case(search, CaseMatching::Smart);
    let receiver = self.options_to_receiver(search);
    receiver
      .iter()
      .filter_map(|sk| fuzzy_engine.match_item(sk.clone()).map(|mr| (sk, mr)))
      .sorted_by_cached_key(|(_, mr)| mr.rank.iter().sum::<i32>())
      .take(self.config.get().result_count)
      .map(|(sk, _)| { 
        (*sk).as_any()
          .downcast_ref::<SkimmableOption>()
          .unwrap()
          .clone()
      })
      .chain(
        self
          .plugins
          .always_present(search)
          .iter()
          .flat_map(|pl| pl.static_items())
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
    Ok(OkAction { value, close_win: true, copy: true }) => {
      app_handle.clipboard_manager().write_text(value.to_string().trim_matches('"')).unwrap();
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
