use crate::{apps::app_convert, plugin::PluginName, plugin_manager::PluginManager};
use anyhow::Context;
use egui::{Color32, FontFamily, Margin, Rounding};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
  collections::HashMap,
  fs,
  ops::Deref,
  path::{Path, PathBuf},
};
use tracing::{error, info};

use crate::template::Template;

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
  pub apps: AppsConfig,
  pub calc: CalcConfig,
  pub webq: WebqConfig,
  app_styles_path: PathBuf,
}

fn default_result_count() -> usize {
  9
}

fn default_hotkey() -> String {
  "CmdOrCtrl+Space".into()
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct WebqConfig {
  pub default_searcher: Option<SearchConfig>,
  pub searchers: HashMap<String, SearchConfig>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SearchConfig {
  pub label: String,
  pub shortname: String,
  pub template: Template,
  pub icon: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct CalcConfig {
  pub prefix: String,
}

impl Default for CalcConfig {
  fn default() -> Self {
    CalcConfig { prefix: "=".into() }
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AppsConfig {
  pub app_paths: Vec<PathBuf>,
  pub app_extension: String,
}

#[derive(Default)]
pub struct AppCache {
  cache_file: PathBuf,
  inner: RwLock<AppCacheInner>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct AppCacheInner {
  pub app_icons: HashMap<String, String>,
}

impl AppCache {
  pub fn load(file: PathBuf) -> Result<Self, anyhow::Error> {
    let config = if !file.exists() {
      info!(
        "File missing, generating default at {}",
        file.to_string_lossy()
      );
      let item = Self {
        cache_file: file,
        ..Self::default()
      };
      item.persist()?;
      item
    } else {
      let inner: AppCacheInner = toml::from_str(&fs::read_to_string(&file)?)?;
      Self {
        cache_file: file,
        inner: RwLock::new(inner),
      }
    };
    Ok(config)
  }

  fn persist(&self) -> Result<(), anyhow::Error> {
    fs::write(&self.cache_file, toml::to_string(&*self.inner.read())?)?;
    Ok(())
  }

  pub fn get_app_icon(&self, updated: &Path) -> Result<String, anyhow::Error> {
    let key = updated.to_str().unwrap().to_string();
    // TODO Tricky deadlock, but ideally we would persist newly discovered items
    //      rather than waiting until next restart
    Ok(
      self
        .inner
        .read()
        .app_icons
        .get(&key)
        .map(|v| v.to_string())
        .get_or_insert_with(|| app_convert::to_icon(updated).unwrap_or_default())
        .clone(),
    )
  }

  pub fn update_app_icons(&self, updated: Vec<PathBuf>) -> Result<(), anyhow::Error> {
    let new_app_icons = {
      let inner = self.inner.read();
      let mut new_app_icons: HashMap<String, String> = updated
        .iter()
        .map(|p| (p.to_str().unwrap().to_string(), p))
        .filter(|(k, _)| !inner.app_icons.contains_key(k))
        .map(|(k, p)| (k, app_convert::to_icon(p).unwrap_or_default()))
        .collect();
      if new_app_icons.is_empty() {
        return Ok(());
      }
      inner.app_icons.iter().for_each(|(k, v)| {
        new_app_icons.insert(k.clone(), v.clone());
      });
      new_app_icons
    };
    self.inner.write().app_icons = new_app_icons;
    self.persist()
  }
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
