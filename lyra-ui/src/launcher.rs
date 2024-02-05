use std::sync::Arc;

use crate::{config::Config, plugin_manager::PluginManager};
use arboard::Clipboard;
use lyra_plugin::{FuzzyMatchItem, OkAction, PluginName};
use nucleo_matcher::{
  pattern::{CaseMatching, Pattern},
  Config as NucleoConfig, Matcher,
};
use parking_lot::RwLock;
use serde_json::Value;

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
  pub fn get_options(&self, search: &str) -> Vec<FuzzyMatchItem> {
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

  pub fn launch(&self, plugin: PluginName, selected: Value) -> Result<OkAction, anyhow::Error> {
    self.plugins.get(&plugin).and_then(|pl| pl.action(selected))
  }
}

pub fn search(launcher: &Launcher, search: &String) -> Vec<(PluginName, Value)> {
  launcher
    .get_options(search)
    .iter()
    .map(|sk| (sk.source.clone(), sk.value.clone()))
    .collect()
}

pub fn submit(
  clipboard: &mut Clipboard,
  launcher: &Launcher,
  for_plugin: PluginName,
  selected: Value,
  close_app: impl FnOnce(),
) -> Result<Value, anyhow::Error> {
  match launcher.launch(for_plugin, selected) {
    Ok(OkAction {
      value,
      close_win: true,
      copy: true,
    }) => {
      clipboard.set_text(value.to_string().trim_matches('"'))?;
      close_app();
      Ok(value)
    }
    Ok(OkAction {
      value,
      close_win: true,
      ..
    }) => {
      close_app();
      Ok(value)
    }
    Ok(OkAction { value, .. }) => Ok(value),
    Err(err) => Err(err),
  }
}
