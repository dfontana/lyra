use std::{collections::HashMap, sync::Arc};
use serde_json::Value;
use skim::SkimItem;

#[derive(Clone)]
pub struct SkimmableOption {
  pub value: Value,
  pub skim: Arc<dyn SkimItem>,
  pub source: PluginName,
}

impl SkimItem for SkimmableOption {
    fn text(&self) -> std::borrow::Cow<str> {
        self.skim.text()
    }
}

pub struct OkAction {
  pub value: Value,
  pub close_win: bool,
  pub copy: bool,
}

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

  /// Execute this plugin against the given input (specific to this plugin). This plugin
  /// will have to deserialize the value to determine what to do with it. If there's an issue 
  /// executing this action a serializable error can be returned (like for Calc this will be)
  /// an object the UI can parse, but for others it might be nothing or just a String.
  ///
  /// Plugins can choose to close the window after they are done executing by setting the boolean
  /// in the returned OkAction.
  fn action(&self, input: Value) -> Result<OkAction, Value>; 

  /// This is the options a plugin wants to contribute based on the given search string. Note this won't
  /// have anything like a prefix on the value, so bear that in mind - it's safe to interpret as is.
  fn skim(&self, search: &str) -> Vec<SkimmableOption>; 

  /// Indicates this plugin has static items it wants to always contribute such as the WebQ plugin,
  /// which always wants to affix serching the web.
  fn has_static_items(&self) -> bool {
    false
  }

  /// Any options that should always be present in the search reguardless of the search should come from
  /// here. This will affix them after skimming.
  fn static_items(&self) -> Vec<SkimmableOption> {
    vec![] 
  }
}
