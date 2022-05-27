use std::{
  collections::HashMap,
  fs,
  path::PathBuf,
  sync::{Arc, Mutex},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Clone, Default)]
pub struct Config {
  pub config: Arc<Mutex<InnerConfig>>,
  file: PathBuf,
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct InnerConfig {
  #[serde(serialize_with = "toml::ser::tables_last")]
  pub bookmarks: HashMap<String, Bookmark>,
  #[serde(serialize_with = "toml::ser::tables_last")]
  pub searchers: HashMap<String, Searcher>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Bookmark {
  pub label: String,
  pub shortname: String,
  pub link: String,
  pub icon: String,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Searcher {
  pub label: String,
  pub shortname: String,
  pub template_link: String,
  pub arg_count: usize,
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
      let mut config = Config::default();
      config.file = conf_file;
      config.persist()?;
      config
    } else {
      let inner: InnerConfig = toml::from_str(&fs::read_to_string(&conf_file)?)?;
      Config {
        config: Arc::new(Mutex::new(inner)),
        file: conf_file,
      }
    };

    Ok(config)
  }

  pub fn update_bookmarks(&self, bookmarks: Vec<Bookmark>) -> Result<(), anyhow::Error> {
    (*self.config.lock().unwrap()).bookmarks =
      bookmarks.iter().fold(HashMap::new(), |mut acc, v| {
        acc.insert(v.label.clone(), v.clone());
        acc
      });
    self.persist()
  }

  fn persist(&self) -> Result<(), anyhow::Error> {
    let inner = self.config.lock().unwrap();
    fs::write(&self.file, toml::to_string(&*inner)?)?;
    Ok(())
  }

  pub fn get_url_from_label(&self, label: &str) -> String {
    if let Some(bookmark) = (*self.config.lock().unwrap()).bookmarks.get(label) {
      bookmark.link.to_owned()
    } else {
      "".to_owned()
    }
  }
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
          .with_writer(tracing_appender::rolling::daily(logs_dir, "lyra.log"))
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
