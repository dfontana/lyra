use std::{path::PathBuf, collections::HashMap};

use calc::Context;
use lyra_plugin::{OkAction, Plugin, SkimmableOption};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct CalcError {
  message: String,
  start: usize,
  end: usize,
}

// TODO: I think we can generalize this to integrate with the launcher::submit
//   where we customize the launch() behavior via the plugin-prefix
//   and customize how the launcher returns results based on prefix-match
//   (eg first checking if any plugins match a prefix, otherwise invoking
//   normal behavior). Resizing behavior may be tricky. This could remove the ui
//   code specific to this
//
// #[tauri::command]
// pub fn calculate(
//   config: tauri::State<'_, Arc<Config>>,
//   window: tauri::Window,
//   expression: String,
// ) -> Result<String, CalcError> {
//   // closer::resize_to(&window, (*config).clone(), 2).map_err(|err| CalcError {
//   //   message: err,
//   //   start: 0,
//   //   end: 0,
//   // })?;
//   eval(&expression)
// }
pub const PLUGIN_NAME: &'static str = "calc";

pub struct CalcPlugin {}

impl CalcPlugin {
  pub fn init(_conf_dir: &PathBuf, _cache_dir: &PathBuf) -> Self {
    todo!()
  }
}

impl Plugin for CalcPlugin {
  fn update_config(&self, _updates: HashMap<String, Value>) -> Result<(), anyhow::Error> {
    todo!()
  }

  fn action(&self, _input: Value) -> Result<OkAction, Value> {
    // TODO: `input` needs to be deserialized for this plugin (right now it's a SearchOption)
    //       The sourced plugin can then execute it's action; which previously looked like this:
    //          let url = self.config.get_url(&selected)?;
    //          open::that(url)?;
    //       Except for calc, we'll instead run the eval method in there
    todo!("Blanket impl for now, should delete once something works")
  }

  fn skim(&self, _search: &str) -> Vec<SkimmableOption> {
    // TODO: Delete this once every plugin impled to prevent compile issues
    todo!("Blanket impl for now, should delete once something works")
  }

fn get_config(&self) -> Value {
        todo!()
    }
}

pub fn eval(search_input: String) -> Result<Value, Value> {
  // TODO: May need to strip prefix here rather than in frontend
  let mut context = Context::<f64>::default();
  context
    .evaluate_annotated(&search_input)
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
    .map_err(|e| serde_json::to_value(e).unwrap())
}
