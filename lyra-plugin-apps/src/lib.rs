use std::{path::PathBuf, collections::HashMap};

use lyra_plugin::{Plugin, OkAction, SkimmableOption};
use serde_json::Value;

mod applookup;
mod config;
mod convert;


pub const PLUGIN_NAME: &'static str = "apps";

pub struct AppsPlugin {}

impl AppsPlugin {
  pub fn init(conf_dir: &PathBuf, cache_dir: &PathBuf) -> Self {
    todo!()
  }
}

impl Plugin for AppsPlugin {
  fn update_config(&self, updates: HashMap<String, Value>) -> Result<(), anyhow::Error> {
    todo!()
  }

  fn action(&self, input: Value) -> Result<OkAction, Value> {
    // TODO: `input` needs to be deserialized for this plugin (right now it's a SearchOption)
    //       The sourced plugin can then execute it's action; which previously looked like this:
    //          let url = self.config.get_url(&selected)?;
    //          open::that(url)?;
    //       Except for calc, we'll instead run the eval method in there
    todo!("Blanket impl for now, should delete once something works")
  }

  fn skim(&self, search: &str) -> Vec<SkimmableOption> {
    // TODO: Delete this once every plugin impled to prevent compile issues
    todo!("Blanket impl for now, should delete once something works")
  }

fn get_config(&self) -> Value {
        todo!()
    }
}

// TODO: impl
// #[derive(Debug, Deserialize, Serialize)]
// pub struct AppOption {
//   pub rank: i32,
//   pub label: String,
//   pub icon: String,
//   pub path: String,
// }

// impl From<App> for SearchOption {
//   fn from(app: App) -> SearchOption {
//     SearchOption::App(AppOption {
//       rank: 0,
//       label: app.label.clone(),
//       path: app.path.to_string_lossy().to_string(),
//       icon: app.icon.clone(),
//     })
//   }
// }

// pub fn get_url(&self, opt: &SearchOption) -> Result<String, anyhow::Error> {
// SearchOption::App(data) => Ok(data.path.clone()),

// Skim Item
//  SearchOption::App(d) => d.label.as_str()
// built from
//  .chain(self.apps.iter().map(App::into))
