mod commands;
mod logs;
mod template;

use crate::{convert, launcher::SearchOption};
use anyhow::{anyhow, Context};
pub use commands::*;
pub use logs::init_logs;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
  collections::HashMap,
  fs,
  ops::Deref,
  path::{Path, PathBuf},
};
use template::Template;
use tracing::{error, info};

#[derive(Debug, Default)]
pub struct Config {
  pub config: RwLock<InnerConfig>,
  file: PathBuf,
  styles_inited: RwLock<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct InnerConfig {
  pub default_web_engine: Option<Template>,
  pub app_paths: Vec<PathBuf>,
  pub app_extension: String,
  pub calc_trigger: String,
  #[serde(default = "default_result_count")]
  pub result_count: usize,
  pub styles: Styles,
  pub app_styles_path: PathBuf,
  #[serde(serialize_with = "toml::ser::tables_last")]
  pub bookmarks: HashMap<String, Bookmark>,
  #[serde(serialize_with = "toml::ser::tables_last")]
  pub searchers: HashMap<String, Searcher>,
  #[serde(serialize_with = "toml::ser::tables_last")]
  pub app_icons: HashMap<String, String>,
}

fn default_result_count() -> usize {
  9
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Styles {
  pub option_width: f64,
  pub option_height: f64,
  pub font_size: usize,
  pub window_placement: Placement,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Placement {
  Center,
  XY(f64, f64),
}

impl Default for Styles {
  fn default() -> Self {
    Self {
      option_width: 600f64,
      option_height: 38f64,
      font_size: 16,
      window_placement: Placement::XY(100.0, 100.0),
    }
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Bookmark {
  pub label: String,
  pub shortname: String,
  pub link: String,
  pub icon: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Searcher {
  pub label: String,
  pub shortname: String,
  pub template: Template,
  pub icon: String,
}

impl Config {
  pub fn get_or_init_config() -> Result<Config, anyhow::Error> {
    let conf_dir = init_home()?;
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
        styles_inited: RwLock::new(false),
      }
    };

    Ok(config)
  }

  pub fn init_styles(&self, defaults_dir: PathBuf, force_write: bool) -> Result<(), anyhow::Error> {
    if *self.styles_inited.read() {
      return Ok(());
    }

    // Initialize all the files that should exist
    info!("Checking for style files");
    let styles_dir = init_home()?.join(&self.get().app_styles_path);
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
    (*self.styles_inited.write()) = true;
    Ok(())
  }

  pub fn get_app_styles_path(&self) -> Result<PathBuf, anyhow::Error> {
    let conf_dir = init_home()?;
    Ok(conf_dir.join(&self.get().app_styles_path))
  }

  pub fn get(&self) -> impl Deref<Target = InnerConfig> + '_ {
    self.config.read()
  }

  pub fn update_bookmarks(&self, updated: Vec<Bookmark>) -> Result<(), anyhow::Error> {
    self.config.write().bookmarks = updated.iter().fold(HashMap::new(), |mut acc, v| {
      acc.insert(v.label.clone(), v.clone());
      acc
    });
    self.persist()
  }

  pub fn update_engine(&self, updated: Template) -> Result<(), anyhow::Error> {
    self.config.write().default_web_engine = Some(updated);
    self.persist()
  }

  pub fn update_searchers(&self, updated: Vec<Searcher>) -> Result<(), anyhow::Error> {
    self.config.write().searchers = updated.iter().fold(HashMap::new(), |mut acc, v| {
      acc.insert(v.label.clone(), v.clone());
      acc
    });
    self.persist()
  }

  pub fn get_app_icon(&self, updated: &Path) -> Result<String, anyhow::Error> {
    let key = updated.to_str().unwrap().to_string();
    // TODO Tricky deadlock, but ideally we would persist newly discovered items
    //      rather than waiting until next restart
    Ok(
      self
        .config
        .read()
        .app_icons
        .get(&key)
        .map(|v| v.to_string())
        .get_or_insert_with(|| convert::to_icon(updated).unwrap_or_default())
        .clone(),
    )
  }

  pub fn update_app_icons(&self, updated: Vec<PathBuf>) -> Result<(), anyhow::Error> {
    let new_app_icons = {
      let inner = self.config.read();
      let mut new_app_icons: HashMap<String, String> = updated
        .iter()
        .map(|p| (p.to_str().unwrap().to_string(), p))
        .filter(|(k, _)| !inner.app_icons.contains_key(k))
        .map(|(k, p)| (k, convert::to_icon(p).unwrap_or_default()))
        .collect();
      if new_app_icons.is_empty() {
        return Ok(());
      }
      inner.app_icons.iter().for_each(|(k, v)| {
        new_app_icons.insert(k.clone(), v.clone());
      });
      new_app_icons
    };
    self.config.write().app_icons = new_app_icons;
    self.persist()
  }

  fn persist(&self) -> Result<(), anyhow::Error> {
    let inner = self.config.read();
    fs::write(&self.file, toml::to_string(&*inner)?)?;
    Ok(())
  }

  pub fn get_url(&self, opt: &SearchOption) -> Result<String, anyhow::Error> {
    match opt {
      SearchOption::App(data) => Ok(data.path.clone()),
      SearchOption::Bookmark(data) => self
        .get()
        .bookmarks
        .get(&data.label)
        .map(|bk| bk.link.clone())
        .ok_or_else(|| anyhow!("No such bookmark")),
      SearchOption::Searcher(data) => self
        .get()
        .searchers
        .get(&data.label)
        .ok_or_else(|| anyhow!("No such searcher"))
        .and_then(|sh| sh.template.hydrate(&data.args).map_err(|err| err.into())),
      SearchOption::WebQuery(query) => self
        .get()
        .default_web_engine
        .as_ref()
        .ok_or_else(|| anyhow!("No WebEngine configured for queries"))
        .and_then(|tp| {
          tp.hydrate(&vec![query.query.clone()])
            .map_err(|err| err.into())
        }),
    }
  }
}

pub fn init_home() -> Result<PathBuf, anyhow::Error> {
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
  Ok(conf_dir)
}
