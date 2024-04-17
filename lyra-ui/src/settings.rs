use std::sync::Arc;

use egui::{Layout, TextEdit, ViewportId, Widget};
use egui_extras::{Column, TableBuilder};
use parking_lot::RwLock;
use tracing::warn;

use crate::config::{Config, Placement};

const LYRA_SETTINGS: &str = "Lyra Settings";

pub struct LyraSettings {
  pub id: ViewportId,
  pub title: String,
  pub visible: Arc<RwLock<bool>>,
  config: Arc<Config>,
  // AppData
  window_x: String,
  window_y: String,
  webq_label: String,
  webq_template: String,
  webq_image: String,
}

impl LyraSettings {
  pub fn new(config: Arc<Config>) -> Self {
    let window_x;
    let window_y;
    {
      let cfg = config.get();
      match cfg.styles.window_placement {
        Placement::XY(x, y) => {
          window_x = x.to_string();
          window_y = y.to_string();
        }
      }
    }
    LyraSettings {
      id: ViewportId::from_hash_of(LYRA_SETTINGS),
      title: LYRA_SETTINGS.into(),
      visible: Arc::new(RwLock::new(false)),
      window_x,
      window_y,
      webq_label: String::new(),
      webq_template: String::new(),
      webq_image: String::new(),
      config,
    }
  }
}

impl LyraSettings {
  pub fn update(&mut self, ctx: &egui::Context) {
    if !*self.visible.read() {
      return;
    }
    ctx.show_viewport_immediate(
      self.id,
      egui::ViewportBuilder::default()
        .with_title(&self.title)
        .with_inner_size([600.0, 500.0]),
      |ctx, _| {
        if ctx.input(|i| i.viewport().close_requested()) {
          *self.visible.write() = false;
        }
        egui::CentralPanel::default().show(ctx, |ui| {
          ui.horizontal_top(|ui| {
            ui.label("Window Placement");
            ui.label("X:");
            ui.add(TextEdit::singleline(&mut self.window_x).desired_width(35.0));
            ui.label("Y:");
            ui.add(TextEdit::singleline(&mut self.window_y).desired_width(35.0));
          });
          ui.separator();
          ui.label("Default Search");
          ui.vertical(|ui| {
            ui.horizontal(|ui| {
              ui.label("Label:");
              ui.add(TextEdit::singleline(&mut self.webq_label).desired_width(200.0));
            });
            ui.horizontal(|ui| {
              ui.label("Template:");
              ui.add(TextEdit::singleline(&mut self.webq_template).desired_width(400.0));
            });
            ui.horizontal(|ui| {
              ui.label("Image:");
              let img = self.webq_image.clone();
              ui.text_edit_singleline(&mut self.webq_image);
              if !img.is_empty() {
                ui.image(img);
              }
            });
          });
          ui.separator();
          ui.horizontal(|ui| {
            TableBuilder::new(ui)
              .striped(true)
              .resizable(false)
              .vscroll(true)
              .auto_shrink(true)
              .column(Column::auto())
              .column(Column::auto())
              .column(Column::auto())
              .column(Column::auto())
              .header(32f32, |mut r| {
                r.col(|ui| {
                  ui.label("Label");
                });
                r.col(|ui| {
                  ui.label("Template");
                });
                r.col(|ui| {
                  ui.label("Image");
                });
                r.col(|ui| {
                  ui.label("Delete");
                });
              })
              .body(|mut body| {
                let rows = 100;
                body.rows(32f32, rows, |mut row| {
                  // TODO: Render row of data
                });
              });
          });
          ui.separator();
          if ui.button("Save").clicked() {
            self.config.update(|mut inner| {
              inner.styles.window_placement = Placement::XY(
                // TODO: Verify user has input a valid value; you might want to make a widget
                self.window_x.parse::<f32>().unwrap(),
                self.window_y.parse::<f32>().unwrap(),
              );
            });
            if let Err(err) = self.config.persist() {
              warn!("Failed to save config update: {}", err);
            }
          }
        });
      },
    );
  }
}

// TODO: How can I make an input that's generic to the underlying type?
trait InputValidator<T> {
  fn validate(inp: T) -> Result<(), String>;
}

struct LabelledInput<T> {
  label: String,
  value: T,
  validator: fn(T) -> Result<(), String>,
}

impl<T> LabelledInput<T> {
  pub fn new_f32(value: f32, label: String) -> Self {
    LabelledInput {
      label,
      value,
      validator: validate_f32,
    }
  }
}

struct F32Input(LabelledInput<f32>);
impl F32Input {
  pub fn new(value: f32, label: String) -> Self {
    Self(LabelledInput {
      label,
      value,
      validator: F32Input::validate,
    })
  }
  pub fn validate(value: f32) -> Result<(), String> {
    todo!()
  }
}

trait Input {
  // fn validate(inp: T) -> Result<(), String>;
}

impl Input for F32Input {}

impl<T> Widget for LabelledInput<T> {
  fn ui(self, ui: &mut egui::Ui) -> egui::Response {
    ui.allocate_ui_with_layout(
      [100.0, 100.0].into(),
      Layout::left_to_right(egui::Align::Min),
      |ui| {
        ui.label("X:");
        ui.add(TextEdit::singleline(&mut self.value).desired_width(35.0));
      },
    )
    .response
  }
}
// impl Widget for NumberInput {
//   fn ui(self, ui: &mut egui::Ui) -> egui::Response {
//     todo!()
//   }
// }
