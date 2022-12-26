use std::{collections::HashMap, fs, path::PathBuf, ops::Deref};

use anyhow::{anyhow, Context};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use crate::template::Template;

#[derive(Default)]
pub struct Config {
  conf_file: PathBuf,
  inner: RwLock<InnerConfig>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct InnerConfig {
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

impl Config {
  pub fn get(&self) -> impl Deref<Target = InnerConfig> + '_ {
    self.inner.read()
  }

  pub fn load(conf_file: PathBuf) -> Result<Self, anyhow::Error> {
    let config = if !conf_file.exists() {
      info!(
        "Config missing, generating default at {}",
        conf_file.to_string_lossy()
      );
      let config = Config {
        conf_file,
        ..Config::default()
      };
      config.persist()?;
      config
    } else {
      let inner: InnerConfig = toml::from_str(&fs::read_to_string(&conf_file)?)?;
      Config {
        conf_file,
        inner: RwLock::new(inner),
      }
    };
    Ok(config)
  }

  pub fn update(&self, updates: HashMap<String, Value>) -> Result<(), anyhow::Error> {
    for (k, v) in updates {
      let status = match k.as_ref() {
        "default_searcher" => self.update_engine(&v),
        "searchers" => self.update_searchers(&v),
        _ => Err(anyhow!("Unknown field")),
      };
      status.context(format!("Field failed operation: {}", k))?;
    }
    self.persist()
  }

  fn update_engine(&self, updated: &Value) -> Result<(), anyhow::Error> {
    let updated = serde_json::from_value::<SearchConfig>(updated.clone())?;
    self.inner.write().default_searcher = Some(updated);
    Ok(())
  }

  fn update_searchers(&self, updated: &Value) -> Result<(), anyhow::Error> {
    let updated = serde_json::from_value::<Vec<SearchConfig>>(updated.clone())?;
    self.inner.write().searchers = updated.iter().fold(HashMap::new(), |mut acc, v| {
      acc.insert(v.label.clone(), v.clone());
      acc
    });
    Ok(())
  }

  fn persist(&self) -> Result<(), anyhow::Error> {
    fs::write(&self.conf_file, toml::to_string(&*self.inner.read())?)?;
    Ok(())
  }
}
