use crate::config::CalcConfig;
use crate::plugin::{
  AppState, FuzzyMatchItem, OkAction, Plugin, PluginV, PluginValue, Renderable, SearchBlocker,
};
use anyhow::anyhow;
use arboard::Clipboard;
use calc::Context;
use egui::{Color32, RichText};
use parking_lot::Mutex;
use std::sync::Arc;

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
  cfg: CalcConfig,
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
  pub fn init(cfg: CalcConfig, clip: Clipboard) -> Result<Self, anyhow::Error> {
    Ok(CalcPlugin {
      cfg,
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

  fn prefix(&self) -> Option<String> {
    Some(self.cfg.prefix.clone())
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
