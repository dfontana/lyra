use serde::Serialize;

use crate::config::Config;
use std::sync::Arc;

pub struct Launcher {
  config: Arc<Config>,
}

#[derive(Serialize)]
pub struct SearchOption {
  id: usize,
  label: String,
}

impl Launcher {
  pub fn new(config: Arc<Config>) -> Self {
    Launcher { config }
  }

  pub async fn get_options(&self, _search: &str) -> Vec<SearchOption> {
    vec![]
  }

  pub fn launch(&self, _id: usize) -> Result<(), anyhow::Error> {
    Ok(())
  }
}
