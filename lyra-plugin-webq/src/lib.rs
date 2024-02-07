use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::Arc};

use anyhow::anyhow;
use config::{SearchConfig, WebqConf};
use egui::{Image, RichText, Ui};
use lyra_common::convert;
use lyra_plugin::{
  Config, FuzzyMatchItem, Launchable, OkAction, Plugin, PluginValue, Renderable, SearchBlocker,
};
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

// TODO Fill these in
impl PluginValue for Searcher {}
impl SearchBlocker for Searcher {
  fn blocks_search(&self, _state: &lyra_plugin::AppState) -> bool {
    // TODO: Use to check if templating is active (complete or started)
    false
  }
}
impl Launchable for Searcher {}
impl Renderable for Searcher {
  fn render(&self, ui: &mut Ui, _state: &lyra_plugin::AppState) {
    // TODO: We can render something better that shows the state of the
    //       hydrated template
    let icon = convert::parse_image_data(&self.icon)
      .ok_or(anyhow!("TODO: Non-PNG support"))
      .and_then(|(s, ext)| convert::decode_bytes(&s).map(|b| (b, ext)));

    ui.horizontal(|ui| {
      if let Ok((img, ext)) = icon {
        ui.add(
          Image::from_bytes(format!("bytes://{}.{}", self.label.to_string(), ext), img)
            .maintain_aspect_ratio(true)
            .shrink_to_fit(),
        );
      }
      ui.label(RichText::new(&self.label));
    });
  }
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

  // TODO: Combine this into the try_launch method
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
      .map(|_| OkAction { close_win: true })
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
      against: Arc::new(searcher.shortname.clone()),
      value: Box::new(searcher),
      source: PLUGIN_NAME.to_string(),
    }
  }
}
fn get_shortname<'a>(v: &'a Value) -> Option<&'a str> {
  v.as_object()
    .and_then(|m| m.get("shortname"))
    .and_then(|v| v.as_str())
}

fn get_required_args(v: &Value) -> Option<usize> {
  v.as_object()
    .and_then(|m| m.get("required_args"))
    .and_then(|v| v.as_u64())
    .map(|v| v as usize)
}

fn is_default_search(shortname: &str) -> bool {
  shortname.is_empty()
}

fn is_bookmark(v: &Value) -> bool {
  get_required_args(v).filter(|n| *n == 0).is_some()
}

fn extract_args<'a>(v: &'a Value, input: &'a str) -> Option<(Vec<&'a str>, usize)> {
  if get_shortname(v)
    .filter(|sn| is_default_search(sn))
    .and(Some(input).filter(|i| !i.is_empty()))
    .is_some()
  {
    return Some((vec![input], 0));
  }
  get_required_args(v).map(|rq| {
    (
      input.trim().splitn(rq + 1, ' ').skip(1).collect::<Vec<_>>(),
      rq,
    )
  })
}

#[derive(Debug, PartialEq, Eq)]
enum TemplatingState {
  NotStarted,
  Started,
  Complete,
}

impl TemplatingState {
  fn templating(&self) -> bool {
    *self != TemplatingState::NotStarted
  }

  fn is_complete(&self) -> bool {
    *self == TemplatingState::Complete
  }

  fn compute(&mut self, selected: Option<&(String, Value)>, input: &str) -> bool {
    match self {
      TemplatingState::NotStarted => {
        if self.is_templating_started(selected, input) {
          *self = TemplatingState::Started;
          return true;
        } else if self.is_templating_complete(selected, input) {
          *self = TemplatingState::Complete;
        }
      }
      TemplatingState::Started => {
        if !self.is_templating_started(selected, input) {
          *self = TemplatingState::NotStarted;
        } else if self.is_templating_complete(selected, input) {
          *self = TemplatingState::Complete;
        }
      }
      TemplatingState::Complete => {
        if !self.is_templating_complete(selected, input) {
          *self = TemplatingState::Started;
        }
      }
    };
    false
  }

  fn is_templating_started(&self, selected: Option<&(String, Value)>, input: &str) -> bool {
    let Some((_, pv)) = selected else {
      return false;
    };
    if is_bookmark(pv) {
      return false;
    }
    let Some(sn) = get_shortname(pv) else {
      return false;
    };
    !is_default_search(sn) && input.starts_with(sn) && input.contains(" ")
  }

  fn is_templating_complete(&self, selected: Option<&(String, Value)>, input: &str) -> bool {
    let Some((_, pv)) = selected else {
      return false;
    };
    if is_bookmark(pv) {
      return false;
    }
    extract_args(pv, input)
      .filter(|(args, required)| args.len() == *required)
      .is_some()
  }
}
