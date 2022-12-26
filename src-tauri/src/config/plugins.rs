use abi_stable::std_types::*;
use lyra_plugin::plugin_internal::{load_plugin, Plugin};
use std::{collections::HashMap, fs, path::Path};
use tracing::{error, warn};

use super::PluginConf;

#[derive(Debug)]
pub struct LoadedPlugin {
  pub plugin: Plugin,
  pub prefix: String,
  pub source: String,
}

pub fn init_plugins(
  config_dir: &Path,
  plugins: &HashMap<String, PluginConf>,
) -> HashMap<String, LoadedPlugin> {
  let mut loaded: HashMap<String, LoadedPlugin> = HashMap::new();
  for (name, plugin) in plugins {
    // TODO: change extension based on platform; dll (win), so (lx), dylib (mac)
    let path = config_dir.join(format!("{}.dylib", name));
    if !path.exists() {
      warn!("Plugin not found, skipping: {}", path.to_string_lossy());
      continue;
    }

    // Create the data dir if it's missing
    let path_data = config_dir.join(name);
    if !path_data.exists() {
      if let Err(e) = fs::create_dir_all(&path_data) {
        error!(
          "Failed to init plugin dir {}, skipping plugin: {}",
          path_data.to_string_lossy(),
          e
        );
        continue;
      }
    }

    let loaded_plugin = match unsafe { load_plugin(&path.to_string_lossy()) } {
      Ok(pd) => pd,
      Err(e) => {
        error!("Failed to load plugin {}: {}", name, e);
        continue;
      }
    };

    let init_result = unsafe {
      loaded_plugin.invoke_init(
        &RString::from(path_data.to_string_lossy()),
        &plugin
          .config
          .iter()
          .map(|(k, v)| (RString::from(k.to_string()), RString::from(v.to_string())))
          .collect(),
      )
    };
    if let RErr(e) = init_result {
      error!("Plugin '{}' failed to initialize {}", name, e);
      continue;
    }

    loaded.insert(
      name.to_string(),
      LoadedPlugin {
        plugin: loaded_plugin,
        prefix: plugin.prefix.clone(),
        source: name.clone(),
      },
    );
  }
  loaded
}
