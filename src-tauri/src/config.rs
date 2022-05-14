use std::{fs, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
  pub bookmarks: Vec<Bookmark>,
  pub searches: Vec<Searcher>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct Bookmark {}

#[derive(Default, Deserialize, Serialize)]
pub struct Searcher {}

pub fn get_or_init_config() -> Result<Config, anyhow::Error> {
  let conf_dir = init_home()?;
  let conf_file = conf_dir.join("config.toml");
  let config = if !conf_file.exists() {
    info!(
      "Config missing, generating default at {}",
      conf_file.to_string_lossy()
    );
    let config = Config::default();
    fs::write(conf_file, toml::to_string(&config)?)?;
    config
  } else {
    toml::from_str(&fs::read_to_string(conf_file)?)?
  };

  Ok(config)
}

pub fn init_logs() -> Result<(), anyhow::Error> {
  tracing::subscriber::with_default(
    FmtSubscriber::builder()
      .with_max_level(Level::INFO)
      .finish(),
    || -> Result<(), anyhow::Error> {
      let logs_dir = init_home()?.join("logs");
      if !logs_dir.exists() {
        info!(
          "Log dir missing, generating default at {}",
          logs_dir.to_string_lossy()
        );
        fs::create_dir_all(&logs_dir)?;
      }
      tracing::subscriber::set_global_default(
        FmtSubscriber::builder()
          .with_max_level(Level::INFO)
          .with_writer(tracing_appender::rolling::hourly(logs_dir, "lyra.log"))
          .finish(),
      )
      .expect("setting default subscriber failed");
      Ok(())
    },
  )
}

fn init_home() -> Result<PathBuf, anyhow::Error> {
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
