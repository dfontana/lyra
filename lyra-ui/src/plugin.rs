use crate::apps::{self, AppLaunch, AppsPlugin};
use crate::calc::{self, CalcPlugin, Evaluated};
use crate::webq::{self, Searcher, WebqPlugin};
use anyhow::anyhow;
use egui::Ui;
use std::{fmt, sync::Arc};

pub enum Plugins {
  Apps(AppsPlugin),
  Calc(CalcPlugin),
  Webq(WebqPlugin),
}

#[derive(Clone)]
pub enum PluginV {
  Apps(AppLaunch),
  Calc(Evaluated),
  Webq(Searcher),
}

impl Plugins {
  pub fn id(&self) -> PluginName {
    match self {
      Plugins::Apps(_) => apps::PLUGIN_NAME.to_owned(),
      Plugins::Calc(_) => calc::PLUGIN_NAME.to_owned(),
      Plugins::Webq(_) => webq::PLUGIN_NAME.to_owned(),
    }
  }

  pub fn validate_value(&self, input_type: &str, input_value: &str) -> Result<(), anyhow::Error> {
    match self {
      Plugins::Apps(pi) => pi.validate_value(input_type, input_value),
      Plugins::Calc(pi) => pi.validate_value(input_type, input_value),
      Plugins::Webq(pi) => pi.validate_value(input_type, input_value),
    }
  }

  pub fn prefix(&self) -> Option<String> {
    match self {
      Plugins::Apps(pi) => pi.prefix(),
      Plugins::Calc(pi) => pi.prefix(),
      Plugins::Webq(pi) => pi.prefix(),
    }
  }

  pub fn action(&self, input: &PluginV) -> Result<OkAction, anyhow::Error> {
    match (self, input) {
      (Plugins::Apps(pi), PluginV::Apps(v)) => pi.action(v),
      (Plugins::Calc(pi), PluginV::Calc(v)) => pi.action(v),
      (Plugins::Webq(pi), PluginV::Webq(v)) => pi.action(v),
      _ => Err(anyhow!("Incompatible plugin and value given")),
    }
  }

  pub fn derive_state(&self, state: &AppState) -> Option<AppState> {
    match self {
      Plugins::Apps(pi) => pi.derive_state(state),
      Plugins::Calc(pi) => pi.derive_state(state),
      Plugins::Webq(pi) => pi.derive_state(state),
    }
  }

  pub fn options(&self, search: &str) -> Vec<FuzzyMatchItem> {
    match self {
      Plugins::Apps(pi) => pi.options(search),
      Plugins::Calc(pi) => pi.options(search),
      Plugins::Webq(pi) => pi.options(search),
    }
  }

  pub fn has_static_items(&self) -> bool {
    match self {
      Plugins::Apps(pi) => pi.has_static_items(),
      Plugins::Calc(pi) => pi.has_static_items(),
      Plugins::Webq(pi) => pi.has_static_items(),
    }
  }

  pub fn static_items(&self) -> Vec<FuzzyMatchItem> {
    match self {
      Plugins::Apps(pi) => pi.static_items(),
      Plugins::Calc(pi) => pi.static_items(),
      Plugins::Webq(pi) => pi.static_items(),
    }
  }
}

impl PluginV {
  pub fn id(&self) -> PluginName {
    match self {
      PluginV::Apps(_) => apps::PLUGIN_NAME.to_owned(),
      PluginV::Calc(_) => calc::PLUGIN_NAME.to_owned(),
      PluginV::Webq(_) => webq::PLUGIN_NAME.to_owned(),
    }
  }

  pub fn render(&self, ui: &mut Ui, state: &AppState) {
    match self {
      PluginV::Apps(v) => v.render(ui, state),
      PluginV::Calc(v) => v.render(ui, state),
      PluginV::Webq(v) => v.render(ui, state),
    }
  }

  pub fn blocks_search(&self, state: &AppState) -> bool {
    match self {
      PluginV::Apps(v) => v.blocks_search(state),
      PluginV::Calc(v) => v.blocks_search(state),
      PluginV::Webq(v) => v.blocks_search(state),
    }
  }
}

#[derive(Clone, Default)]
pub struct AppState {
  pub input: String,
  pub options: Vec<PluginV>,
  pub selected: usize,
}

impl AppState {
  pub fn selected(&self) -> Option<&PluginV> {
    self.options.get(self.selected)
  }
}

pub struct FuzzyMatchItem {
  pub value: PluginV,
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

pub trait SearchBlocker {
  /// Determines if further searching should be prevented, because this
  /// plugin is waiting for further input instead
  fn blocks_search(&self, _state: &AppState) -> bool {
    false
  }
}

pub trait Renderable {
  /// Implement this to customize how your plugin renders it's data in the UI,
  /// which will be positioned below the main search bar. This is ran for each
  /// plugin result, so your data will be displayed alongside other plugin data.
  /// The ui context is shared across plugins
  fn render(&self, ui: &mut Ui, state: &AppState);
}

pub trait PluginValue: SearchBlocker + Renderable {}

pub type PluginName = String;
pub trait Plugin: Send + Sync {
  type PV: PluginValue;

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

  /// Execute this plugin against the given input (specific to this plugin).
  /// Plugins can choose to close the window after they are done executing by setting the boolean
  /// in the returned OkAction.
  fn action(&self, input: &Self::PV) -> Result<OkAction, anyhow::Error>;

  /// If this plugin wants to manipulate the state of the app, this is a hook
  /// to do so whenever the state changes.
  fn derive_state(&self, _state: &AppState) -> Option<AppState> {
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
