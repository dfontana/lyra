use crate::config::{WebqConfig, WebqSearchConfig};
use crate::icon_ui::Icon;
use crate::plugin::{
  AppState, FuzzyMatchItem, OkAction, Plugin, PluginV, PluginValue, Renderable, SearchBlocker,
};
use crate::template::Template;
use anyhow::anyhow;
use egui::{RichText, Ui};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};

pub const PLUGIN_NAME: &'static str = "webq";

/// This Plugin encompasses 3 types of functionality since they all are just specializations of one another.
/// The core idea is to open a weblink based on some parameterization. So, we have 3 behaviors:
///   1. A "search" against a templated string
///   2. A static, always viable option to search the web when there's no matches. This is just 1 against a
///      specific search engine.
///   3. A "bookmark" which is just a template that has no arguments
/// All of this should be generalizable over the first case, hence the single plugin
pub struct WebqPlugin {
  cfg: WebqConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Searcher {
  Bookmark(Metadata, String),
  Template(Metadata, TemplateData),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Metadata {
  pub label: String,
  pub shortname: String,
  pub icon: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TemplateData {
  pub template: Template,
  #[serde(skip_serializing, skip_deserializing)]
  args: Vec<String>,
  #[serde(skip_serializing, skip_deserializing)]
  state: TemplatingState,
}

impl PluginValue for Searcher {}
impl SearchBlocker for Searcher {
  fn blocks_search(&self, state: &AppState) -> bool {
    state.options.iter().any(|opt| match opt {
      PluginV::Webq(s) => s.is_non_default_templating(),
      _ => false,
    })
  }
}

impl Renderable for Searcher {
  fn render(&self, ui: &mut Ui, _state: &AppState) {
    let md = self.metadata();
    ui.horizontal(|ui| {
      if let Ok(ico) = Icon::try_from((md.icon.as_str(), md.label.as_str())) {
        ico.render(ui);
      }
      if let Searcher::Template(_, td) = self {
        ui.label(RichText::new(&format!(
          "{}: {}",
          md.label,
          td.template.partial_hydrate(&td.args)
        )));
      } else {
        ui.label(RichText::new(&md.label));
      }
    });
  }
}

impl WebqPlugin {
  pub fn init(cfg: WebqConfig) -> Result<Self, anyhow::Error> {
    Ok(WebqPlugin { cfg })
  }
}

impl Plugin for WebqPlugin {
  type PV = Searcher;

  fn validate_value(&self, input_type: &str, input_value: &str) -> Result<(), anyhow::Error> {
    match input_type {
      "template" => Template::from_str(input_value)
        .map(|_| ())
        .map_err(|e| e.into()),
      _ => Err(anyhow!("Unknown input type: {}", input_type)),
    }
  }

  fn derive_state(&self, state: &AppState) -> Option<AppState> {
    let templates: Vec<PluginV> = state
      .options
      .clone()
      .into_iter()
      .map(|opt| match opt {
        PluginV::Webq(s) => match s.update(&state.input) {
          Some(su) => PluginV::Webq(su),
          None => PluginV::Webq(s),
        },
        x => x,
      })
      .collect();

    let is_templating = templates.iter().any(|opt| match opt {
      PluginV::Webq(s) => s.is_non_default_templating(),
      _ => false,
    });

    let mut new_state: AppState = (*state).clone();
    if is_templating {
      // TODO this will move selection every time templates
      // update if multiple match the same prefix or if you type space after partially matching
      // the prefix. should compute this to instead keep selected item if it's a templating item
      // otherwise default to 0 with the first one matching
      new_state.selected = 0;
      new_state.options = templates
        .into_iter()
        .filter(|opt| match opt {
          PluginV::Webq(s) => s.is_non_default_templating(),
          _ => false,
        })
        .collect();
    } else {
      new_state.options = templates;
    }
    Some(new_state)
  }

  fn action(&self, input: &Searcher) -> Result<OkAction, anyhow::Error> {
    if let Searcher::Template(md, ts) = input {
      if !ts.state.is_complete() {
        return Ok(OkAction {
          close_win: false,
          update_input: Some(format!("{} ", md.shortname)),
        });
      }
    }
    let md = input.metadata();
    let target: Result<String, anyhow::Error> = match input {
      Searcher::Bookmark(_, target) => Ok(target.to_string()),
      Searcher::Template(_, ts) => ts.template.hydrate(&ts.args).map_err(|err| err.into()),
    };
    target
      .and_then(|url| open::that(url).map_err(|err| err.into()))
      .map(|_| OkAction {
        close_win: true,
        ..Default::default()
      })
      .map_err(|err| anyhow!("Action failed for {:?}, err: {:?}", md.label, err))
  }

  fn options(&self, _: &str) -> Vec<FuzzyMatchItem> {
    self.cfg.searchers.iter().map(|(_, sh)| sh.into()).collect()
  }

  fn has_static_items(&self) -> bool {
    true
  }

  fn static_items(&self) -> Vec<FuzzyMatchItem> {
    match &self.cfg.default_searcher {
      Some(sh) => vec![sh.into()],
      None => vec![],
    }
  }
}

impl From<&WebqSearchConfig> for Searcher {
  fn from(sh: &WebqSearchConfig) -> Searcher {
    let md = Metadata {
      label: sh.label.clone(),
      shortname: sh.shortname.clone(),
      icon: sh.icon.clone(),
    };
    match sh.template.markers == 0 {
      true => Searcher::Bookmark(md, (*sh.template).to_string()),
      false => Searcher::Template(
        md,
        TemplateData {
          template: sh.template.clone(),
          args: Vec::default(),
          state: TemplatingState::default(),
        },
      ),
    }
  }
}

impl From<&WebqSearchConfig> for FuzzyMatchItem {
  fn from(sh: &WebqSearchConfig) -> Self {
    let searcher = Into::<Searcher>::into(sh);
    FuzzyMatchItem {
      against: Arc::new(searcher.metadata().shortname.clone()),
      value: PluginV::Webq(searcher),
      source: PLUGIN_NAME.to_string(),
    }
  }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
enum TemplatingState {
  #[default]
  NotStarted,
  Started,
  Complete,
}

impl Metadata {
  fn is_default(&self) -> bool {
    self.shortname.is_empty()
  }
}

impl Searcher {
  fn is_non_default_templating(&self) -> bool {
    match self {
      Searcher::Bookmark(_, _) => false,
      Searcher::Template(md, ts) => ts.state.templating() && !md.is_default(),
    }
  }

  fn metadata(&self) -> &Metadata {
    match self {
      Searcher::Bookmark(md, _) => md,
      Searcher::Template(md, _) => md,
    }
  }

  fn update(&self, inp: &String) -> Option<Searcher> {
    let (md, td) = match self {
      Searcher::Bookmark(_, _) => return None,
      Searcher::Template(md, td) => (md, td),
    };
    let sn = md.shortname.as_str();
    if md.is_default() {
      let args = Some(inp)
        .filter(|i| !i.is_empty())
        .map(|s| vec![s.to_string()]);
      return match args {
        Some(args) => Some(Searcher::Template(
          md.clone(),
          TemplateData {
            state: TemplatingState::Complete,
            args,
            ..td.clone()
          },
        )),
        None => Some(Searcher::Template(
          md.clone(),
          TemplateData {
            state: TemplatingState::NotStarted,
            args: Vec::new(),
            ..td.clone()
          },
        )),
      };
    }

    let (state, args) = match inp.split_once(" ") {
      Some((p, _)) if p == sn => {
        let args: Vec<String> = inp
          .trim()
          .splitn(td.template.markers + 1, ' ')
          .skip(1)
          .map(|s| s.to_owned())
          .collect();
        let state = if args.len() == td.template.markers {
          TemplatingState::Complete
        } else {
          TemplatingState::Started
        };
        (state, args)
      }
      None | Some(_) => (TemplatingState::NotStarted, Vec::new()),
    };

    if td.state != state || td.args.len() != args.len() || td.args != args {
      // Updates detected
      return Some(Searcher::Template(
        md.clone(),
        TemplateData {
          state,
          args,
          ..td.clone()
        },
      ));
    }
    None
  }
}

impl TemplatingState {
  fn templating(&self) -> bool {
    *self != TemplatingState::NotStarted
  }

  fn is_complete(&self) -> bool {
    *self == TemplatingState::Complete
  }
}
