use crate::{
  apps::{self, AppsPlugin},
  calc::{self, CalcPlugin},
  config::Config,
  plugin::{OkAction, PluginName, PluginV, Plugins},
  webq::{self, WebqPlugin},
};
use anyhow::anyhow;
use arboard::Clipboard;
use std::{
  collections::{hash_map::Values, HashMap},
  sync::Arc,
};

#[derive(Clone)]
pub struct PluginManager(Arc<HashMap<PluginName, Plugins>>);

impl PluginManager {
  pub fn init(config: &Arc<Config>) -> Result<Self, anyhow::Error> {
    let cfg = config.get();
    let plugs: Result<HashMap<_, _>, _> = cfg
      .plugins
      .iter()
      .map(|pn| {
        let pl = match pn.as_str() {
          calc::PLUGIN_NAME => {
            Plugins::Calc(CalcPlugin::init(cfg.calc.clone(), Clipboard::new()?)?)
          }
          webq::PLUGIN_NAME => Plugins::Webq(WebqPlugin::init(cfg.webq.clone())?),
          apps::PLUGIN_NAME => {
            Plugins::Apps(AppsPlugin::init(cfg.apps.clone(), &config.cache_dir)?)
          }
          _ => return Err(anyhow!("{} is an unknown plugin", pn)),
        };
        Ok((pl.id(), pl))
      })
      .collect();
    Ok(PluginManager(Arc::new(plugs?)))
  }

  pub fn try_launch(&mut self, opt: &PluginV) -> Result<OkAction, anyhow::Error> {
    self
      .0
      .get(&opt.id())
      .ok_or_else(|| anyhow!("Unknown plugin given"))
      .and_then(|pls| pls.action(opt))
  }

  pub fn get(&self, plug: &PluginName) -> Result<&Plugins, anyhow::Error> {
    self
      .0
      .get(plug)
      .ok_or_else(|| anyhow!("Plugin {} not found", plug))
  }

  pub fn iter(&self) -> Values<'_, String, Plugins> {
    self.0.values()
  }

  /// Return the plugins whose prefix are found within the search string, or if none
  /// are found, then return everything
  pub fn filter_to(&self, search: &str) -> Vec<&Plugins> {
    let plugs: Vec<_> = self
      .0
      .values()
      .filter(|pl| match pl.prefix() {
        None => false,
        Some(pre) => search.starts_with(&pre),
      })
      .collect();

    if plugs.is_empty() {
      // Everything BUT the prefixed items
      return self.0.values().filter(|pl| pl.prefix().is_none()).collect();
    }

    plugs
  }

  /// Return the plugins that has something static they want to contribute
  pub fn always_present(&self, search: &str) -> Vec<&Plugins> {
    // Note: optimization here would be to pass a state between filter_to and here
    //       so we don't need to re-check if any prefixes matched
    if self.0.values().any(|pl| match pl.prefix() {
      None => false,
      Some(pre) => search.starts_with(&pre),
    }) {
      return Vec::new();
    }

    self.0.values().filter(|pl| pl.has_static_items()).collect()
  }
}
