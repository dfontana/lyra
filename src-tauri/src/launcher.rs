use serde::Serialize;

use crate::config::Config;

pub struct Launcher {
  config: Config,
}

#[derive(Serialize)]
pub struct SearchOption {
  id: usize,
  label: String,
  icon: String,
}

impl Launcher {
  pub fn new(config: Config) -> Self {
    Launcher { config }
  }

  pub async fn get_options(&self, _search: &str) -> Vec<SearchOption> {
    // TODO actually search the config for options
    (*self.config.config.lock().unwrap())
      .bookmarks
      .iter()
      .enumerate()
      .map(|(id, (_, bk))| SearchOption {
        id,
        label: bk.label.clone(),
        icon: bk.icon.clone(),
      })
      .collect()
  }

  pub fn launch(&self, _id: usize) -> Result<(), anyhow::Error> {
    // TODO need to store the ID against each option during runtime, so we can find them faster/const
    Ok(())
  }
}
