use super::Config;
use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub struct CalcConf(pub Config<InnerConfig>);

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct InnerConfig {
  pub prefix: String,
}

impl CalcConf {
  pub fn update(&self, updates: HashMap<String, Value>) -> Result<(), anyhow::Error> {
    for (k, v) in updates {
      let status = match k.as_ref() {
        "prefix" => self.update_prefix(&v),
        _ => Err(anyhow!("Unknown field")),
      };
      status.context(format!("Field failed operation: {}", k))?;
    }
    self.0.persist()
  }

  fn update_prefix(&self, updated: &Value) -> Result<(), anyhow::Error> {
    let updated = serde_json::from_value::<String>(updated.clone())?;
    self.0.inner.write().prefix = updated;
    Ok(())
  }
}
