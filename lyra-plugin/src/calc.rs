mod config;
use crate::{
  AppState, Config, FuzzyMatchItem, OkAction, Plugin, PluginV, PluginValue, Renderable,
  SearchBlocker,
};
use anyhow::anyhow;
use arboard::Clipboard;
use calc::Context;
use config::CalcConf;
use egui::{Color32, RichText};
use parking_lot::Mutex;
use serde_json::Value;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

pub const PLUGIN_NAME: &'static str = "calc";

#[derive(Clone)]
pub enum Evaluated {
  Ok(String),
  Err {
    message: String,
    start: usize,
    end: usize,
  },
}

pub struct CalcPlugin {
  cfg: CalcConf,
  clip: Mutex<Clipboard>,
}

impl PluginValue for Evaluated {}
impl Renderable for Evaluated {
  fn render(&self, ui: &mut egui::Ui, state: &AppState) {
    ui.horizontal(|ui| match self {
      Evaluated::Ok(v) => {
        ui.label(RichText::new(v));
        return;
      }
      Evaluated::Err {
        message,
        start,
        end,
      } => match (start, end, message) {
        (s, e, _) if *s != 0 && *e != 0 => {
          let inp = &state.input;
          ui.label(RichText::new(&inp[1..*s]));
          ui.label(RichText::new(&inp[*s..*e + 1]).color(Color32::RED));
          ui.label(RichText::new(&inp[*e + 1..]));
        }
        (_, _, message) => {
          ui.label(RichText::new(message));
        }
      },
    });
  }
}
impl SearchBlocker for Evaluated {}

impl CalcPlugin {
  pub fn init(conf_dir: &PathBuf, _: &PathBuf, clip: Clipboard) -> Result<Self, anyhow::Error> {
    let cfg = Config::load(conf_dir.join(format!("{}.toml", PLUGIN_NAME)))?;
    Ok(CalcPlugin {
      cfg: CalcConf(cfg),
      clip: Mutex::new(clip),
    })
  }

  fn eval(&self, search_input: &str) -> Evaluated {
    let mut context = Context::<f64>::default();
    let res = context
      .evaluate_annotated(&search_input.strip_prefix(&self.prefix().unwrap()).unwrap())
      .map_err(|err| match err {
        calc::Error::Parse(err) => match err {
          lalrpop_util::ParseError::InvalidToken { location } => Evaluated::Err {
            message: "Invalid Token".into(),
            start: location,
            end: location,
          },
          lalrpop_util::ParseError::UnrecognizedEOF { location, .. } => Evaluated::Err {
            message: "Unfinished Expression".into(),
            start: location,
            end: location,
          },
          lalrpop_util::ParseError::UnrecognizedToken {
            token: (start, _token, end),
            ..
          } => Evaluated::Err {
            message: "Unknown Token".into(),
            start,
            end,
          },
          lalrpop_util::ParseError::ExtraToken {
            token: (start, _token, end),
            ..
          } => Evaluated::Err {
            message: "Extra Token".into(),
            start,
            end,
          },
          lalrpop_util::ParseError::User { error } => Evaluated::Err {
            message: format!("Unknown Error {}", error),
            start: 0,
            end: 0,
          },
        },
        _ => Evaluated::Err {
          message: format!("Unknown Error {}", err),
          start: 0,
          end: 0,
        },
      });
    match res {
      Ok(v) => Evaluated::Ok(v),
      Err(e) => e,
    }
  }
}

impl Plugin for CalcPlugin {
  type PV = Evaluated;

  fn get_config(&self) -> Value {
    serde_json::to_value((*self.cfg.0.get()).clone()).unwrap()
  }

  fn update_config(&self, updates: HashMap<String, Value>) -> Result<(), anyhow::Error> {
    self.cfg.update(updates)
  }

  fn prefix(&self) -> Option<String> {
    Some(self.cfg.0.get().prefix.clone())
  }

  fn action(&self, input: &Evaluated) -> Result<OkAction, anyhow::Error> {
    match input {
      Evaluated::Ok(v) => self
        .clip
        .lock()
        .set_text(v)
        .map(|_| OkAction { close_win: true })
        .map_err(|e| anyhow!(e)),
      Evaluated::Err { .. } => Err(anyhow!("Can't take action on invalid calculation")),
    }
  }

  fn options(&self, search: &str) -> Vec<FuzzyMatchItem> {
    vec![FuzzyMatchItem {
      value: PluginV::Calc(self.eval(search)),
      against: Arc::new(search.to_owned()),
      source: PLUGIN_NAME.to_string(),
    }]
  }
}
