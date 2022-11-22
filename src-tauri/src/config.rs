mod commands;
mod logs;
mod template;

pub use commands::*;
pub use logs::init_logs;

use crate::launcher::SearchOption;
use anyhow::{anyhow, Context};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, ops::Deref, path::PathBuf};
use template::Template;
use tracing::info;

#[derive(Debug, Default)]
pub struct Config {
  pub config: RwLock<InnerConfig>,
  file: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct InnerConfig {
  pub default_web_engine: Option<Template>,
  pub app_paths: Vec<PathBuf>,
  pub app_extension: String,
  pub styles: Styles,
  #[serde(serialize_with = "toml::ser::tables_last")]
  pub bookmarks: HashMap<String, Bookmark>,
  #[serde(serialize_with = "toml::ser::tables_last")]
  pub searchers: HashMap<String, Searcher>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Styles {
  pub option_width: f64,
  pub option_height: f64,
  pub font_size: usize,
}

impl Default for Styles {
  fn default() -> Self {
    Self {
      option_width: 600f64,
      option_height: 38f64,
      font_size: 16,
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
      }
    };

    Ok(config)
  }

  pub fn get(&self) -> impl Deref<Target = InnerConfig> + '_ {
    self.config.read()
  }

  pub fn update_bookmarks(&self, updated: Vec<Bookmark>) -> Result<(), anyhow::Error> {
    (*self.config.write()).bookmarks = updated.iter().fold(HashMap::new(), |mut acc, v| {
      acc.insert(v.label.clone(), v.clone());
      acc
    });
    self.persist()
  }

  pub fn update_engine(&self, updated: Template) -> Result<(), anyhow::Error> {
    (*self.config.write()).default_web_engine = Some(updated);
    self.persist()
  }

  pub fn update_searchers(&self, updated: Vec<Searcher>) -> Result<(), anyhow::Error> {
    (*self.config.write()).searchers = updated.iter().fold(HashMap::new(), |mut acc, v| {
      acc.insert(v.label.clone(), v.clone());
      acc
    });
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
        .ok_or(anyhow!("No such bookmark")),
      SearchOption::Searcher(data) => self
        .get()
        .searchers
        .get(&data.label)
        .ok_or(anyhow!("No such searcher"))
        .and_then(|sh| sh.template.hydrate(&data.args).map_err(|err| err.into())),
      SearchOption::WebQuery(query) => self
        .get()
        .default_web_engine
        .as_ref()
        .ok_or(anyhow!("No WebEngine configured for queries"))
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
    .or(
      std::env::var("HOME")
        .map(PathBuf::from)
        .map(|p| p.join(".config/lyra")),
    )
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
