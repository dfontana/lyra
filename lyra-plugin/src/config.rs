use std::{fs, ops::Deref, path::PathBuf};

use parking_lot::RwLock;
use serde::ser::Serialize;
use toml::macros::Deserialize;
use tracing::info;

#[derive(Default)]
pub struct Config<IN>
where
  IN: Default,
{
  conf_file: PathBuf,
  pub inner: RwLock<IN>,
}

impl<IN> Config<IN>
where
  for<'a> IN: Default + Deserialize<'a> + Serialize,
{
  pub fn get(&self) -> impl Deref<Target = IN> + '_ {
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
      let inner: IN = toml::from_str(&fs::read_to_string(&conf_file)?)?;
      Self {
        conf_file,
        inner: RwLock::new(inner),
      }
    };
    Ok(config)
  }

  pub fn persist(&self) -> Result<(), anyhow::Error> {
    fs::write(&self.conf_file, toml::to_string(&*self.inner.read())?)?;
    Ok(())
  }
}
