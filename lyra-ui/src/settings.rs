use std::sync::Arc;

use egui::{Color32, Layout, Stroke, TextEdit, ViewportId, Widget};
use egui_extras::{Column, TableBuilder};
use form_macro::FormResult;
use parking_lot::RwLock;
use tracing::warn;

use crate::{
  config::{Config, Placement, WebqSearchConfig},
  template::Template,
};

const LYRA_SETTINGS: &str = "Lyra Settings";

#[derive(FormResult)]
pub struct LyraSettings {
  pub id: ViewportId,
  pub title: String,
  pub visible: Arc<RwLock<bool>>,
  config: Arc<Config>,
  // AppData
  window_x: FieldData<f32>,
  window_y: FieldData<f32>,
  webq_label: FieldData<String>,
  // TODO use Template type
  webq_template: FieldData<String>,
  webq_image: FieldData<String>,
}

impl LyraSettings {
  pub fn new(config: Arc<Config>) -> Self {
    let window_x;
    let window_y;
    {
      let cfg = config.get();
      match cfg.styles.window_placement {
        Placement::XY(x, y) => {
          window_x = x;
          window_y = y;
        }
      }
    }
    LyraSettings {
      id: ViewportId::from_hash_of(LYRA_SETTINGS),
      title: LYRA_SETTINGS.into(),
      visible: Arc::new(RwLock::new(false)),
      window_x: FieldData::new(window_x),
      window_y: FieldData::new(window_y),
      webq_label: FieldData::new(String::new()),
      webq_template: FieldData::new(String::new()),
      webq_image: FieldData::new(String::new()),
      config,
    }
  }
}

impl LyraSettings {
  fn validate_xy(v: &f32) -> Result<(), String> {
    if *v < 0.0 {
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
              Input::f32("X:", &mut self.window_x)
                .validated(&LyraSettings::validate_xy)
                .desired_width(35.0),
            );
            ui.add(
              Input::f32("Y:", &mut self.window_y)
                .validated(&LyraSettings::validate_xy)
                .desired_width(35.0),
            );
          });
          ui.separator();
          ui.label("Default Search");
          ui.vertical(|ui| {
            ui.add(Input::str("Label:", &mut self.webq_label).desired_width(200.0));
            ui.add(Input::str("Template:", &mut self.webq_template).desired_width(400.0));
            ui.horizontal(|ui| {
              let img = self
                .webq_image
                .value
                .clone()
                .unwrap_or_else(|_| String::new());
              ui.add(Input::str("Image:", &mut self.webq_image).desired_width(400.0));
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
            if let Ok(res) = TryInto::<LyraSettingsFormResult>::try_into(&*self) {
              self.config.update(move |mut inner| {
                inner.styles.window_placement = Placement::XY(res.window_x, res.window_y);
                inner.webq.default_searcher = Some(WebqSearchConfig {
                  label: res.webq_label,
                  shortname: "".into(),
                  template: Template::default(),
                  icon: res.webq_image,
                })
                // TODO: Add more fields
                // inner.webq.searchers;
                // inner.apps.app_paths;
                // inner.apps.app_extension;
                // inner.calc.prefix;
              })
            }
            if let Err(err) = self.config.persist() {
              warn!("Failed to save config update: {}", err);
            }
          }
        });
      },
    );
  }
}

// TODO: Separate crate https://stackoverflow.com/questions/73691794/expose-struct-generated-from-quote-macro-without-appearing-out-of-nowhere
struct FieldData<T: Clone> {
  buffer: String,
  value: Result<T, String>,
}

impl<T: ToString + Clone> FieldData<T> {
  pub fn new(value: T) -> FieldData<T> {
    let buffer = value.to_string();
    FieldData {
      value: Ok(value),
      buffer,
    }
  }
}

struct Input<'a, T: Clone> {
  label: &'a str,
  field: &'a mut FieldData<T>,
  parser: Box<dyn Fn(&String) -> Result<T, String>>,
  validator: Box<dyn Fn(&T) -> Result<(), String>>,
  desired_width: Option<f32>,
}

impl<'a> Input<'a, String> {
  pub fn str(label: &'a str, field: &'a mut FieldData<String>) -> Input<'a, String> {
    Input {
      label,
      field,
      desired_width: None,
      parser: Box::new(|v| Ok(v.to_owned())),
      validator: Box::new(|_| Ok(())),
    }
  }

  pub fn f32(label: &'a str, field: &'a mut FieldData<f32>) -> Input<'a, f32> {
    Input {
      label,
      field,
      desired_width: None,
      parser: Box::new(|v| {
        v.parse::<f32>()
          .map_err(|_| format!("{} is not a number", v))
      }),
      validator: Box::new(|_| Ok(())),
    }
  }
}

impl<'a, T: Clone> Input<'a, T> {
  #[inline]
  fn desired_width(mut self, v: f32) -> Self {
    self.desired_width = Some(v);
    self
  }

  #[inline]
  fn validated(mut self, validator: impl Fn(&T) -> Result<(), String> + 'static) -> Self {
    self.validator = Box::new(validator);
    self
  }

  fn parse_and_validate(self) {
    self.field.value = (self.parser)(&self.field.buffer);
    if let Ok(v) = self.field.value.as_ref() {
      if let Err(err) = (self.validator)(&v) {
        self.field.value = Err(err);
      }
    }
  }
}

impl<'a, T: Clone> Widget for Input<'a, T> {
  fn ui(self, ui: &mut egui::Ui) -> egui::Response {
    ui.allocate_ui_with_layout(
      ui.available_size(),
      Layout::left_to_right(egui::Align::Min),
      |ui| {
        ui.label(self.label);
        if self.field.value.is_err() {
          let invalid = Stroke::new(1.0, Color32::RED);
          ui.style_mut().visuals.widgets.inactive.bg_stroke = invalid;
          ui.style_mut().visuals.widgets.hovered.bg_stroke = invalid;
          ui.style_mut().visuals.selection.stroke = invalid;
        };
        let mut edit = ui.add(
          TextEdit::singleline(&mut self.field.buffer)
            .desired_width(self.desired_width.unwrap_or(f32::INFINITY)),
        );
        if let Err(err) = &self.field.value {
          edit = edit.on_hover_text(err);
        }
        if edit.changed() {
          self.parse_and_validate();
        }
      },
    )
    .response
  }
}
