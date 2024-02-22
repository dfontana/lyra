use super::template::Template;
use super::Config;
use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub struct WebqConf(pub Config<InnerConfig>);

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

impl WebqConf {
  pub fn update(&self, updates: HashMap<String, Value>) -> Result<(), anyhow::Error> {
    for (k, v) in updates {
      let status = match k.as_ref() {
        "default_searcher" => self.update_engine(&v),
        "searchers" => self.update_searchers(&v),
        _ => Err(anyhow!("Unknown field")),
      };
      status.context(format!("Field failed operation: {}", k))?;
    }
    self.0.persist()
  }

  fn update_engine(&self, updated: &Value) -> Result<(), anyhow::Error> {
    let updated = serde_json::from_value::<SearchConfig>(updated.clone())?;
    self.0.inner.write().default_searcher = Some(updated);
    Ok(())
  }

  fn update_searchers(&self, updated: &Value) -> Result<(), anyhow::Error> {
    let updated = serde_json::from_value::<Vec<SearchConfig>>(updated.clone())?;
    self.0.inner.write().searchers = updated.iter().fold(HashMap::new(), |mut acc, v| {
      acc.insert(v.label.clone(), v.clone());
      acc
    });
    Ok(())
  }
}
