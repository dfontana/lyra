mod config;

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use calc::Context;
use config::CalcConf;
use lyra_plugin::{
  Config, FuzzyMatchItem, Launchable, OkAction, Plugin, PluginValue, Renderable, SearchBlocker,
};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct CalcError {
  message: String,
  start: usize,
  end: usize,
}

pub enum Evaluated {
  Ok(String),
  Err(CalcError),
}

// TODO Fill these in
impl PluginValue for Evaluated {}
impl Renderable for Evaluated {}
impl SearchBlocker for Evaluated {}
impl Launchable for Evaluated {}

pub const PLUGIN_NAME: &'static str = "calc";

pub struct CalcPlugin {
  cfg: CalcConf,
}

impl CalcPlugin {
  pub fn init(conf_dir: &PathBuf, _: &PathBuf) -> Result<Self, anyhow::Error> {
    let cfg = Config::load(conf_dir.join(format!("{}.toml", PLUGIN_NAME)))?;
    Ok(CalcPlugin { cfg: CalcConf(cfg) })
  }

  fn eval(&self, search_input: &str) -> Evaluated {
    let mut context = Context::<f64>::default();
    context
      .evaluate_annotated(&search_input.strip_prefix(&self.prefix().unwrap()).unwrap())
      .map_err(|err| match err {
        calc::Error::Parse(err) => match err {
          lalrpop_util::ParseError::InvalidToken { location } => CalcError {
            message: "Invalid Token".into(),
            start: location,
            end: location,
          },
          lalrpop_util::ParseError::UnrecognizedEOF { location, .. } => CalcError {
            message: "Unfinished Expression".into(),
            start: location,
            end: location,
          },
          lalrpop_util::ParseError::UnrecognizedToken {
            token: (start, _token, end),
            ..
          } => CalcError {
            message: "Unknown Token".into(),
            start,
            end,
          },
          lalrpop_util::ParseError::ExtraToken {
            token: (start, _token, end),
            ..
          } => CalcError {
            message: "Extra Token".into(),
            start,
            end,
          },
          lalrpop_util::ParseError::User { error } => CalcError {
            message: format!("Unknown Error {}", error),
            start: 0,
            end: 0,
          },
        },
        _ => CalcError {
          message: format!("Unknown Error {}", err),
          start: 0,
          end: 0,
        },
      })
      .map(|v| serde_json::to_value(v).unwrap())
      .map_err(|e| serde_json::to_value(e).unwrap());
    // TODO refactor return of this method
    todo!()
  }
}

impl Plugin for CalcPlugin {
  fn get_config(&self) -> Value {
    serde_json::to_value((*self.cfg.0.get()).clone()).unwrap()
  }

  fn update_config(&self, updates: HashMap<String, Value>) -> Result<(), anyhow::Error> {
    self.cfg.update(updates)
  }

  fn prefix(&self) -> Option<String> {
    Some(self.cfg.0.get().prefix.clone())
  }

  fn action(&self, input: Value) -> Result<OkAction, anyhow::Error> {
    // TODO: Copy value onto clipboard
    Ok(OkAction { close_win: true })
  }

  fn options(&self, search: &str) -> Vec<FuzzyMatchItem> {
    vec![FuzzyMatchItem {
      value: Box::new(self.eval(search)),
      against: Arc::new(search.to_owned()),
      source: PLUGIN_NAME.to_string(),
    }]
  }
}
