use parking_lot::{RwLock, RwLockWriteGuard};
use serde::{de::DeserializeOwned, Serialize};
use std::{fs, ops::Deref, path::PathBuf};
use tracing::info;

#[derive(Debug, Default)]
pub struct Cache<C: DeserializeOwned + Serialize + Default> {
  cache_file: PathBuf,
  inner: RwLock<C>,
}

impl<C: DeserializeOwned + Serialize + Default> Cache<C> {
  pub fn blank(file: PathBuf) -> Self {
    Cache {
      cache_file: file,
      ..Self::default()
    }
  }

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
      let inner: C = toml::from_str(&fs::read_to_string(&file)?)?;
      Self {
        cache_file: file,
        inner: RwLock::new(inner),
      }
    };
    Ok(config)
  }

  pub fn get(&self) -> impl Deref<Target = C> + '_ {
    self.inner.read()
  }

  pub fn update<'a>(&'a self, func: impl FnOnce(RwLockWriteGuard<'a, C>)) {
    let inner = self.inner.write();
    func(inner);
  }

  pub fn persist(&self) -> Result<(), anyhow::Error> {
    let inner = self.inner.read();
    fs::write(&self.cache_file, toml::to_string(&*inner)?)?;
    Ok(())
  }
}
