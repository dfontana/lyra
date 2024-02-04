use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::Arc};

use anyhow::anyhow;
use config::{SearchConfig, WebqConf};
use lyra_plugin::{Config, FuzzyMatchItem, OkAction, Plugin};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use template::Template;

mod config;
mod template;

pub const PLUGIN_NAME: &'static str = "webq";

/// This Plugin encompasses 3 types of functionality since they all are just specializations of one another.
/// The core idea is to open a weblink based on some parameterization. So, we have 3 behaviors:
///   1. A "search" against a templated string
///   2. A static, always viable option to search the web when there's no matches. This is just 1 against a
///      specific search engine.
///   3. A "bookmark" which is just a template that has no arguments
/// All of this should be generalizable over the first case, hence the single plugin
pub struct WebqPlugin {
  cfg: WebqConf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Searcher {
  pub label: String,
  pub shortname: String,
  pub icon: String,
  pub required_args: usize,
  pub args: Vec<String>,
}

impl WebqPlugin {
  pub fn init(conf_dir: &PathBuf, _: &PathBuf) -> Result<Self, anyhow::Error> {
    let cfg = Config::load(conf_dir.join(format!("{}.toml", PLUGIN_NAME)))?;
    Ok(WebqPlugin { cfg: WebqConf(cfg) })
  }
}

impl Plugin for WebqPlugin {
  fn get_config(&self) -> Value {
    serde_json::to_value((*self.cfg.0.get()).clone()).unwrap()
  }

  fn update_config(&self, updates: HashMap<String, Value>) -> Result<(), anyhow::Error> {
    self.cfg.update(updates)
  }

  fn validate_value(&self, input_type: &str, input_value: &str) -> Result<(), anyhow::Error> {
    match input_type {
      "template" => Template::from_str(input_value)
        .map(|_| ())
        .map_err(|e| e.into()),
      _ => Err(anyhow!("Unknown input type: {}", input_type)),
    }
  }

  fn action(&self, input: Value) -> Result<lyra_plugin::OkAction, anyhow::Error> {
    let data: Searcher = serde_json::from_value(input)?;
    let cfg = self.cfg.0.get();
    cfg
      .searchers
      .get(&data.label)
      .or_else(|| {
        // Check if it's the default real quick before bailing
        cfg
          .default_searcher
          .as_ref()
          .filter(|sc| sc.label == data.label)
      })
      .ok_or_else(|| anyhow!("No such searcher"))
      .and_then(|sh| sh.template.hydrate(&data.args).map_err(|err| err.into()))
      .and_then(|url| open::that(url).map_err(|err| err.into()))
      .map(|_| OkAction {
        value: Value::Null,
        close_win: true,
        copy: false,
      })
      .map_err(|err| anyhow!("Action failed for {:?}, err: {:?}", data.label, err))
  }

  fn options(&self, _: &str) -> Vec<lyra_plugin::FuzzyMatchItem> {
    self
      .cfg
      .0
      .get()
      .searchers
      .iter()
      .map(|(_, sh)| sh.into())
      .collect()
  }

  fn has_static_items(&self) -> bool {
    true
  }

  fn static_items(&self) -> Vec<lyra_plugin::FuzzyMatchItem> {
    match &self.cfg.0.get().default_searcher {
      Some(sh) => vec![sh.into()],
      None => vec![],
    }
  }
}

impl From<&SearchConfig> for Searcher {
  fn from(sh: &SearchConfig) -> Searcher {
    Searcher {
      label: sh.label.clone(),
      shortname: sh.shortname.clone(),
      icon: sh.icon.clone(),
      required_args: sh.template.markers,
      args: Vec::new(),
    }
  }
}

impl From<&SearchConfig> for FuzzyMatchItem {
  fn from(sh: &SearchConfig) -> Self {
    let searcher = Into::<Searcher>::into(sh);
    FuzzyMatchItem {
      value: serde_json::to_value(&searcher).unwrap(),
      against: Arc::new(searcher.shortname.clone()),
      source: PLUGIN_NAME.to_string(),
    }
  }
}
