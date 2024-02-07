use std::sync::Arc;

use crate::{config::Config, plugin_manager::PluginManager};
use lyra_plugin::{FuzzyMatchItem, PluginValue};
use nucleo_matcher::{
  pattern::{CaseMatching, Pattern},
  Config as NucleoConfig, Matcher,
};
use parking_lot::RwLock;

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
      .into_iter()
      .take(self.config.get().result_count)
      .map(|(v, _)| v)
      .chain(
        self
          .plugins
          .always_present(search)
          .iter()
          .flat_map(|pl| pl.static_items()),
      )
      .collect()
  }
}

pub fn search(launcher: &Launcher, search: &String) -> Vec<Box<dyn PluginValue>> {
  launcher
    .get_options(search)
    .into_iter()
    .map(|sk| sk.value)
    .collect()
}
