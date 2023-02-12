use anyhow::{anyhow, Context};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fs, ops::Deref, path::PathBuf};
use tracing::info;

#[derive(Default)]
pub struct Config {
  conf_file: PathBuf,
  inner: RwLock<InnerConfig>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct InnerConfig {
  pub prefix: String,
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
      let config = Self {
        conf_file,
        ..Self::default()
      };
      config.persist()?;
      config
    } else {
      let inner: InnerConfig = toml::from_str(&fs::read_to_string(&conf_file)?)?;
      Self {
        conf_file,
        inner: RwLock::new(inner),
      }
    };
    Ok(config)
  }

  fn persist(&self) -> Result<(), anyhow::Error> {
    fs::write(&self.conf_file, toml::to_string(&*self.inner.read())?)?;
    Ok(())
  }

  pub fn update(&self, updates: HashMap<String, Value>) -> Result<(), anyhow::Error> {
    for (k, v) in updates {
      let status = match k.as_ref() {
        "prefix" => self.update_prefix(&v),
        _ => Err(anyhow!("Unknown field")),
      };
      status.context(format!("Field failed operation: {}", k))?;
    }
    self.persist()
  }

  fn update_prefix(&self, updated: &Value) -> Result<(), anyhow::Error> {
    let updated = serde_json::from_value::<String>(updated.clone())?;
    self.inner.write().prefix = updated;
    Ok(())
  }
}
