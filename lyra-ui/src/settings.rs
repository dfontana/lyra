use std::sync::Arc;

use egui::{Color32, Layout, Stroke, TextEdit, ViewportId, Widget};
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
  fn validate_xy(v: f32) -> Result<(), String> {
    if v < 0.0 {
      return Err("Must be larger than 0".into());
    }
    Ok(())
  }

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
            ui.add(
              // TODO: Store a ref to this input and extract value on save
              //       Or is there a better way to gather the form :thinking:
              Input::new("X:", &mut self.window_x)
                .f32(&LyraSettings::validate_xy)
                .desired_width(35.0),
            );
            ui.add(
              Input::new("Y:", &mut self.window_y)
                .f32(&LyraSettings::validate_xy)
                .desired_width(35.0),
            );
          });
          ui.separator();
          ui.label("Default Search");
          ui.vertical(|ui| {
            ui.add(Input::new("Label:", &mut self.webq_label).desired_width(200.0));
            ui.add(Input::new("Template:", &mut self.webq_template).desired_width(400.0));
            ui.horizontal(|ui| {
              let img = self.webq_image.clone();
              ui.add(Input::new("Image:", &mut self.webq_image).desired_width(400.0));
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

struct Input<'a, T> {
  label: &'a str,
  value: &'a mut String,
  validator: Box<dyn Fn(&String) -> Result<T, String>>,
  desired_width: Option<f32>,
}

impl<'a> Input<'a, String> {
  pub fn new(label: &'a str, value: &'a mut String) -> Input<'a, String> {
    Input {
      label,
      value,
      desired_width: None,
      validator: Box::new(|v| Ok(v.to_owned())),
    }
  }

  pub fn f32(self, validator: impl Fn(f32) -> Result<(), String> + 'static) -> Input<'a, f32> {
    let wrapped = Box::new(move |v: &String| {
      v.parse::<f32>()
        .map_err(|_| format!("{} is not a number", v))
        .and_then(|v| (validator)(v).map(|_| v))
    });
    Input {
      validator: wrapped,
      label: self.label,
      value: self.value,
      desired_width: self.desired_width,
    }
  }
}

impl<'a, T> Input<'a, T> {
  #[inline]
  fn desired_width(mut self, v: f32) -> Self {
    self.desired_width = Some(v);
    self
  }

  fn validate(&self) -> Result<T, String> {
    (self.validator)(&self.value)
  }

  pub fn value(&self) -> Option<T> {
    self.validate().ok()
  }
}

impl<'a, T> Widget for Input<'a, T> {
  fn ui(self, ui: &mut egui::Ui) -> egui::Response {
    ui.allocate_ui_with_layout(
      ui.available_size(),
      Layout::left_to_right(egui::Align::Min),
      |ui| {
        ui.label(self.label);
        let err = self.validate().err();
        if err.is_some() {
          let invalid = Stroke::new(1.0, Color32::RED);
          ui.style_mut().visuals.widgets.inactive.bg_stroke = invalid;
          ui.style_mut().visuals.widgets.hovered.bg_stroke = invalid;
          ui.style_mut().visuals.selection.stroke = invalid;
        };
        let edit = ui.add(
          TextEdit::singleline(self.value)
            .desired_width(self.desired_width.unwrap_or(f32::INFINITY)),
        );
        if let Some(err) = err {
          edit.on_hover_text(err);
        }
      },
    )
    .response
  }
}
