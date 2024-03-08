use anyhow::Context;
use egui::{Color32, FontFamily, Margin, Rounding};
use lyra_plugin::{PluginManager, PluginName};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fs, ops::Deref, path::PathBuf, sync::Arc};
use tracing::{error, info};

#[derive(Debug, Default)]
pub struct Config {
  pub config: RwLock<InnerConfig>,
  file: PathBuf,
  pub cache_dir: PathBuf,
  pub conf_dir: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct InnerConfig {
  #[serde(default = "default_result_count")]
  pub result_count: usize,
  pub styles: Styles,
  pub plugins: Vec<PluginName>,
  #[serde(default = "default_hotkey")]
  pub hotkey: String,
  app_styles_path: PathBuf,
}

fn default_result_count() -> usize {
  9
}

fn default_hotkey() -> String {
  "CmdOrCtrl+Space".into()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Styles {
  pub window_placement: Placement,
  pub window_size: (f32, f32),
  // TODO: Could make this better validated/integrated. Can it even be parsed?
  pub window_rounding: Rounding,
  pub window_padding: f32,
  pub option_margin: Margin,
  pub option_rounding: Rounding,
  pub bg_color: Color32,
  pub bg_color_selected: Color32,
  pub text_color: Color32,
  pub text_color_selected: Color32,
  pub font_family: FontFamily,
  pub font_size: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Placement {
  XY(f32, f32),
}

impl Default for Styles {
  fn default() -> Self {
    Self {
      window_placement: Placement::XY(100.0, 100.0),
      window_size: (600.0, 32.0),
      window_rounding: 5.0.into(),
      window_padding: 4.0,
      option_margin: 4.0.into(),
      option_rounding: 2.0.into(),
      bg_color: Color32::WHITE,
      bg_color_selected: Color32::from_hex("#54e6ae").unwrap(),
      text_color: Color32::DARK_GRAY,
      text_color_selected: Color32::WHITE,
      font_family: FontFamily::Monospace,
      font_size: 16.0,
    }
  }
}

pub fn init_home() -> Result<(PathBuf, PathBuf), anyhow::Error> {
  let maybe_path = std::env::var("LYRA_HOME")
    .map(PathBuf::from)
    .or_else(|_| {
      std::env::var("HOME")
        .map(PathBuf::from)
        .map(|p| p.join(".config/lyra"))
    })
    .with_context(|| "Must set either LYRA_HOME or HOME for config resolution");

  let conf_dir = maybe_path?;
  if !conf_dir.exists() {
    info!(
      "Config dir missing, generating default at {}",
      conf_dir.to_string_lossy()
    );
    fs::create_dir_all(&conf_dir)?;
  }

  let cache_dir = conf_dir.join("cache");
  if !cache_dir.exists() {
    info!(
      "Cache dir missing, generating default at {}",
      cache_dir.to_string_lossy()
    );
    fs::create_dir_all(&cache_dir)?;
  }

  Ok((conf_dir, cache_dir))
}

impl Config {
  pub fn get_or_init_config() -> Result<Config, anyhow::Error> {
    let (conf_dir, cache_dir) = init_home()?;
    let conf_file = conf_dir.join("config.toml");
    let config = if !conf_file.exists() {
      info!(
        "Config missing, generating default at {}",
        conf_file.to_string_lossy()
      );
      let config = Config {
        file: conf_file,
        ..Config::default()
      };
      config.persist()?;
      config
    } else {
      let inner: InnerConfig = toml::from_str(&fs::read_to_string(&conf_file)?)?;
      Config {
        config: RwLock::new(inner),
        file: conf_file,
        conf_dir,
        cache_dir,
      }
    };

    Ok(config)
  }

  pub fn init_styles(&self, defaults_dir: PathBuf, force_write: bool) -> Result<(), anyhow::Error> {
    // Initialize all the files that should exist
    info!("Checking for style files");
    let (conf_dir, _) = init_home()?;
    let styles_dir = conf_dir.join(&self.get().app_styles_path);
    fs::read_dir(defaults_dir.join("resources"))?
      .filter_map(Result::ok)
      .filter_map(|file| {
        file
          .file_name()
          .as_os_str()
          .to_string_lossy()
          .strip_prefix("default_")
          .map(|name| (file.path(), name.to_string()))
      })
      .map(|(file, file_name)| (file, styles_dir.join(file_name)))
      .filter(|(_, path)| force_write || !path.exists())
      .for_each(|(default_file_path, config_style_path)| {
        info!(
          "Initializing style file {} @ {}",
          default_file_path.display(),
          config_style_path.display()
        );
        if let Err(e) = std::fs::copy(default_file_path, config_style_path) {
          error!("Failed to init style file {:?}", e)
        };
      });
    Ok(())
  }

  pub fn get(&self) -> impl Deref<Target = InnerConfig> + '_ {
    self.config.read()
  }

  fn persist(&self) -> Result<(), anyhow::Error> {
    let inner = self.config.read();
    fs::write(&self.file, toml::to_string(&*inner)?)?;
    Ok(())
  }
}

pub fn get_config(plugin_manager: PluginManager, config: Arc<Config>) -> HashMap<String, Value> {
  let mut configs = plugin_manager.get_configs().clone();
  configs.insert(
    "general".to_string(),
    serde_json::to_value(config.get().clone()).unwrap(),
  );
  configs
}

pub fn save_plugin_settings(
  plugin_manager: PluginManager,
  for_plugin: PluginName,
  updates: HashMap<String, Value>,
) -> Result<(), String> {
  plugin_manager
    .get(&for_plugin) // Result<Plugin, Error>; eg failure to find plugin
    .and_then(|pl| pl.update_config(updates))
    .map_err(|err| {
      error!("Failed to update plugin {}'s config: {}", for_plugin, err);
      "Failed to save changes".into()
    })
}

pub fn validate_plugin_value(
  plugin_manager: PluginManager,
  for_plugin: PluginName,
  input_type: String,
  input_value: String,
) -> Result<(), String> {
  plugin_manager
    .get(&for_plugin)
    .map_err(|err| {
      error!("Failed to verify plugin {} input: {}", for_plugin, err);
      "Error locating plugin to validate with".to_string()
    })
    .and_then(|pl| {
      pl.validate_value(&input_type, &input_value)
        .map_err(|e| format!("{}", e))
    })
}
