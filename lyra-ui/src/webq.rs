use crate::config::{WebqConfig, WebqSearchConfig};
use crate::plugin::{
  AppState, FuzzyMatchItem, OkAction, Plugin, PluginV, PluginValue, Renderable, SearchBlocker,
};
use crate::template::Template;
use anyhow::anyhow;
use egui::{Image, RichText, Ui};
use lyra_common::convert;
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
pub struct Searcher {
  pub label: String,
  pub shortname: String,
  pub icon: String,
  pub required_args: usize,
  #[serde(skip_serializing, skip_deserializing)]
  args: Vec<String>,
  #[serde(skip_serializing, skip_deserializing)]
  state: TemplatingState,
}

impl PluginValue for Searcher {}
impl SearchBlocker for Searcher {
  fn blocks_search(&self, state: &AppState) -> bool {
    state.options.iter().any(|opt| match opt {
      PluginV::Webq(s) => s.state.templating() && !s.is_default(),
      _ => false,
    })
  }
}

impl Renderable for Searcher {
  fn render(&self, ui: &mut Ui, _state: &AppState) {
    // TODO: We can render something better that shows the state of the
    //       hydrated template
    let icon = convert::parse_image_data(&self.icon)
      .ok_or(anyhow!("Cannot render image format"))
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
      PluginV::Webq(s) => s.state.templating() && !s.is_default(),
      _ => false,
    });

    let mut new_state: AppState = (*state).clone();
    if is_templating {
      // TODO this will move selection every time templates
      // update; should compute this to instead keep selected item
      // if it's a templating item otherwise default to 0
      new_state.selected = 0;
      new_state.options = templates
        .into_iter()
        .filter(|opt| match opt {
          PluginV::Webq(s) => s.state.templating() && !s.is_default(),
          _ => false,
        })
        .collect();
    } else {
      new_state.options = templates;
    }
    Some(new_state)
  }

  fn action(&self, input: &Searcher) -> Result<OkAction, anyhow::Error> {
    if !(input.is_bookmark() || input.state.is_complete()) {
      // TODO: Would be nice to enter templating mode on the selected
      //       item, which will require updating the input to be the prefix
      //       + a space & then updating template state
      return Err(anyhow!("Templating is not complete"));
    }
    self
      .cfg
      .searchers
      .get(&input.label)
      .or_else(|| {
        // Check if it's the default real quick before bailing
        self
          .cfg
          .default_searcher
          .as_ref()
          .filter(|sc| sc.label == input.label)
      })
      .ok_or_else(|| anyhow!("No such searcher"))
      .and_then(|sh| sh.template.hydrate(&input.args).map_err(|err| err.into()))
      .and_then(|url| open::that(url).map_err(|err| err.into()))
      .map(|_| OkAction { close_win: true })
      .map_err(|err| anyhow!("Action failed for {:?}, err: {:?}", input.label, err))
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
    Searcher {
      label: sh.label.clone(),
      shortname: sh.shortname.clone(),
      icon: sh.icon.clone(),
      required_args: sh.template.markers,
      args: Vec::default(),
      state: TemplatingState::default(),
    }
  }
}

impl From<&WebqSearchConfig> for FuzzyMatchItem {
  fn from(sh: &WebqSearchConfig) -> Self {
    let searcher = Into::<Searcher>::into(sh);
    FuzzyMatchItem {
      against: Arc::new(searcher.shortname.clone()),
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

impl Searcher {
  fn is_bookmark(&self) -> bool {
    self.required_args == 0
  }

  fn is_default(&self) -> bool {
    self.shortname.is_empty()
  }

  fn update(&self, inp: &String) -> Option<Searcher> {
    if self.is_bookmark() {
      return None;
    }

    let sn = self.shortname.as_str();
    if self.is_default() {
      let args = Some(inp)
        .filter(|i| !i.is_empty())
        .map(|s| vec![s.to_string()]);
      return match args {
        Some(args) => Some(Searcher {
          state: TemplatingState::Complete,
          args,
          ..self.clone()
        }),
        None => Some(Searcher {
          state: TemplatingState::NotStarted,
          args: Vec::new(),
          ..self.clone()
        }),
      };
    }

    let (state, args) = match inp.split_once(" ") {
      Some((p, _)) if p == sn => {
        let args: Vec<String> = inp
          .trim()
          .splitn(self.required_args + 1, ' ')
          .skip(1)
          .map(|s| s.to_owned())
          .collect();
        let state = if args.len() == self.required_args {
          TemplatingState::Complete
        } else {
          TemplatingState::Started
        };
        (state, args)
      }
      None | Some(_) => (TemplatingState::NotStarted, Vec::new()),
    };

    if self.state != state || self.args.len() != args.len() || self.args != args {
      // Updates detected
      return Some(Searcher {
        state,
        args,
        ..self.clone()
      });
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
