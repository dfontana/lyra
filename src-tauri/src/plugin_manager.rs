use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use lyra_plugin::{Plugin, PluginName};
use lyra_plugin_apps::AppsPlugin;
use lyra_plugin_calc::CalcPlugin;
use lyra_plugin_webq::WebqPlugin;
use serde_json::Value;

use crate::config::Config;

#[derive(Clone)]
pub struct PluginManager(Arc<HashMap<PluginName, Box<dyn Plugin>>>);

impl PluginManager {
  pub fn init(cfg: Arc<Config>) -> Result<Self, anyhow::Error> {
    let plugs: Result<HashMap<_, Box<dyn Plugin>>, _> = cfg
      .get()
      .plugins
      .iter()
      .map(|pn| {
        let pl = match pn.as_ref() {
          lyra_plugin_calc::PLUGIN_NAME => {
            Box::new(CalcPlugin::init(&cfg.conf_dir, &cfg.cache_dir)?) as Box<dyn Plugin>
          }
          lyra_plugin_webq::PLUGIN_NAME => {
            Box::new(WebqPlugin::init(&cfg.conf_dir, &cfg.cache_dir)?) as Box<dyn Plugin>
          }
          lyra_plugin_apps::PLUGIN_NAME => {
            Box::new(AppsPlugin::init(&cfg.conf_dir, &cfg.cache_dir)?) as Box<dyn Plugin>
          }
          _ => return Err(anyhow!("{} is an unknown plugin", pn)),
        };

        Ok((pn.clone(), pl))
      })
      .collect();
    Ok(PluginManager(Arc::new(plugs?)))
  }

  pub fn get(&self, plug: &PluginName) -> Result<&Box<dyn Plugin>, anyhow::Error> {
    self
      .0
      .get(plug)
      .ok_or_else(|| anyhow!("Plugin {} not found", plug))
  }

  /// Return all the serialized configs for each plugin so the UI can hydrate settings
  pub fn get_configs(&self) -> HashMap<PluginName, Value> {
    self
      .0
      .iter()
      .map(|(pn, pl)| (pn.clone(), pl.get_config()))
      .collect()
  }

  /// Return the plugins whose prefix are found within the search string, or if none
  /// are found, then return everything
  pub fn filter_to(&self, search: &str) -> Vec<&Box<dyn Plugin>> {
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
  pub fn always_present(&self, search: &str) -> Vec<&Box<dyn Plugin>> {
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
