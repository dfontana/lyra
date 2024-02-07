mod config;

pub use config::*;

use egui::Ui;
use serde_json::Value;
use std::{collections::HashMap, fmt, sync::Arc};

#[derive(Default)]
pub struct AppState {
  pub input: String,
  pub options: Vec<Box<dyn PluginValue>>,
  pub selected: usize,
}

impl AppState {
  pub fn selected(&self) -> Option<Box<dyn PluginValue>> {
    // TODO fixup
    // self.options.get(self.selected)
    todo!()
  }
}

pub struct FuzzyMatchItem {
  pub value: Box<dyn PluginValue>,
  pub against: Arc<dyn AsRef<str>>,
  pub source: PluginName,
}

impl AsRef<str> for FuzzyMatchItem {
  fn as_ref(&self) -> &str {
    self.against.as_ref().as_ref()
  }
}

impl fmt::Debug for FuzzyMatchItem {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "against:{:?},source:{:?}", self.as_ref(), self.source)
  }
}

pub struct OkAction {
  pub close_win: bool,
}

pub trait Launchable {
  /// Execute this plugin against the given input (specific to this plugin).
  /// Plugins can choose to close the window after they are done executing by setting the boolean
  /// in the returned OkAction.
  fn try_launch(&self, state: &AppState) -> Result<OkAction, anyhow::Error> {
    // let value = match pv.as_str() {
    //           "calc" => opt.get("Ok").unwrap().clone(),
    //           "apps" => opt.clone(),
    //           "webq" if self.state.templating.is_complete() || is_bookmark(opt) => {
    //             let (args, _) = extract_args(opt, &self.state.input).unwrap();
    //             opt
    //               .as_object()
    //               .map(|m| {
    //                 let mut m = m.clone();
    //                 m.insert(
    //                   "args".into(),
    //                   Value::Array(args.iter().map(|v| Value::String(v.to_string())).collect()),
    //                 );
    //                 Value::Object(m)
    //               })
    //               .unwrap()
    //           }
    //           "webq" => {
    //             // TODO: Would be nice to enter templating mode on the selected
    //             //       item, which will require updating the input to be the prefix
    //             //       + a space & then updating template state
    //             return;
    //           }
    //           _ => return,
    //         };

    // // TODO: Move this into the calc plugin, not needed here
    // self.clipboard.set_text(value.to_string().trim_matches('"'));
    todo!()
  }
}

pub trait SearchBlocker {
  /// Determines if further searching should be prevented, because this
  /// plugin is waiting for further input instead
  fn blocks_search(&self, _state: &AppState) -> bool {
    false
  }
}

pub trait Renderable {
  // TODO: Use to render each plugin
  fn render(&self, ui: &mut Ui, state: &AppState) {
    // TODO: Move into each plugin
    // match plugin_name.as_str() {
    //             "calc" => mk_calc(ui, opt, &self.state.input),
    //             "apps" => mk_app_res(ui, opt),
    //             "webq" => mk_app_res(ui, opt), // This can be rendered better to show populated template
    //             unk => {
    //               error!("Unknown plugin: {}", unk);
    //             }
    //           };
  }
}

pub trait PluginValue: Launchable + SearchBlocker + Renderable {}

pub type PluginName = String;
pub trait Plugin: Send + Sync {
  /// Return the config object backing this plugin for UI hydration
  fn get_config(&self) -> Value;

  /// With the given updates, plugins should merge their in-memory view of their configuration
  /// and write their config out. If there's a problem validating
  /// the resolved structure or otherwise serializing, an error can be returned.
  fn update_config(&self, updates: HashMap<String, Value>) -> Result<(), anyhow::Error>;

  /// Not all plugins need to have a validation routine, so this method is optional. In some cases,
  /// however, plguins may have specific values that need to be validated either during configuration
  /// or runtime. Based on the uniquely named input_type (to this plugin), it can decide how to validate
  /// the given value. For example, the settings page for searchers will want to validate templates
  /// but bookmarks may not validate anything (or maybe one day check if they are valid URLs).
  fn validate_value(&self, _input_type: &str, _input_value: &str) -> Result<(), anyhow::Error> {
    Ok(())
  }

  /// The Unique Prefix that identifies this plugin, if any at all. For example, a calculator might
  /// prefix itself by having the user type '=' first. But other plugins like app launching or
  /// web searching will have no prefix to trigger their behavior (it just happens naturally)
  fn prefix(&self) -> Option<String> {
    None
  }

  // TODO: Replaced with the Launchable trait -- delete this

  fn action(&self, input: Value) -> Result<OkAction, anyhow::Error>;

  /// If this plugin wants to manipulate the state of the app, this is a hook
  /// to do so whenever the state changes.
  fn derive_state(&self, state: &AppState) -> Option<AppState> {
    None
  }

  /// This is the options a plugin wants to contribute based on the given search string. Note this won't
  /// have anything like a prefix on the value, so bear that in mind - it's safe to interpret as is.
  fn options(&self, search: &str) -> Vec<FuzzyMatchItem>;

  /// Indicates this plugin has static items it wants to always contribute such as the WebQ plugin,
  /// which always wants to affix serching the web.
  fn has_static_items(&self) -> bool {
    false
  }

  /// Any options that should always be present in the search reguardless of the search should come from
  /// here. This will affix them after fuzzy matching.
  fn static_items(&self) -> Vec<FuzzyMatchItem> {
    vec![]
  }
}
