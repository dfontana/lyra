use derive_more::{Display, FromStr};
use egui::{Color32, Layout, Stroke, TextEdit, ViewportId, Widget};
use egui_extras::{Column, TableBuilder};
use form::{FormField, FormFieldData, FormResult, Validate};
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::warn;

use crate::{
  config::{Config, Placement, WebqSearchConfig},
  template::Template,
};

const LYRA_SETTINGS: &str = "Lyra Settings";

pub struct LyraSettings {
  pub id: ViewportId,
  pub title: String,
  pub visible: Arc<RwLock<bool>>,
  config: Arc<Config>,
  form: LyraSettingsForm,
}

#[derive(FormResult, Default)]
struct LyraSettingsForm {
  window_x: FormField<WindowCoordinate>,
  window_y: FormField<WindowCoordinate>,
  webq_label: FormField<WebqLabel>,
  webq_template: FormField<Template>,
  webq_image: FormField<WebqImage>,
}

impl Validate for Template {
  fn validate(_v: &Self) -> Result<(), String> {
    // TODO actually implement this, if there's anything left after parse
    Ok(())
  }
}

#[derive(Clone, Default, Display, FromStr, Validate, FormFieldData)]
struct WebqLabel(String);

#[derive(Clone, Default, Display, FromStr, Validate, FormFieldData)]
struct WebqImage(String);

#[derive(Clone, Default, Display, FormFieldData, FromStr)]
struct WindowCoordinate(f32);
impl Validate for WindowCoordinate {
  fn validate(v: &Self) -> Result<(), String> {
    if v.0 < 0.0 {
      return Err("Must be larger than 0".into());
    }
    Ok(())
  }
}

impl LyraSettings {
  pub fn new(config: Arc<Config>) -> Self {
    let mut form = LyraSettingsForm::default();
    {
      let cfg = config.get();
      match cfg.styles.window_placement {
        Placement::XY(x, y) => {
          form.window_x = FormField::new(WindowCoordinate(x));
          form.window_y = FormField::new(WindowCoordinate(y));
        }
      }
      if let Some(webq) = &cfg.webq.default_searcher {
        form.webq_label = FormField::new(WebqLabel(webq.label.clone()));
        form.webq_template = FormField::new(webq.template.clone());
        form.webq_image = FormField::new(WebqImage(webq.icon.clone()));
      }
    }
    LyraSettings {
      id: ViewportId::from_hash_of(LYRA_SETTINGS),
      title: LYRA_SETTINGS.into(),
      visible: Arc::new(RwLock::new(false)),
      config,
      form,
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
            ui.add(Input::of("X:", &mut self.form.window_x).desired_width(35.0));
            ui.add(Input::of("Y:", &mut self.form.window_y).desired_width(35.0));
          });
          ui.separator();
          ui.label("Default Search");
          ui.vertical(|ui| {
            ui.add(Input::of("Label:", &mut self.form.webq_label).desired_width(200.0));
            ui.add(Input::of("Template:", &mut self.form.webq_template).desired_width(400.0));
            ui.horizontal(|ui| {
              let img = self
                .form
                .webq_image
                .value
                .as_ref()
                .map(|v| v.0.clone())
                .unwrap_or_else(|_| String::new());
              ui.add(Input::of("Image:", &mut self.form.webq_image).desired_width(400.0));
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
            if let Ok(res) = TryInto::<LyraSettingsFormFormResult>::try_into(&self.form) {
              self.config.update(move |mut inner| {
                inner.styles.window_placement = Placement::XY(res.window_x.0, res.window_y.0);
                inner.webq.default_searcher = Some(WebqSearchConfig {
                  label: res.webq_label.0,
                  shortname: "".into(),
                  template: res.webq_template,
                  icon: res.webq_image.0,
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

struct Input<'a, T: FormFieldData> {
  label: &'a str,
  field: &'a mut FormField<T>,
  desired_width: Option<f32>,
}

impl<'a, T: FormFieldData> Input<'a, T> {
  pub fn of(label: &'a str, field: &'a mut FormField<T>) -> Input<'a, T> {
    Input {
      label,
      field,
      desired_width: None,
    }
  }

  #[inline]
  fn desired_width(mut self, v: f32) -> Self {
    self.desired_width = Some(v);
    self
  }
}

impl<'a, T: FormFieldData> Widget for Input<'a, T> {
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
          self.field.parse_and_validate();
        }
      },
    )
    .response
  }
}
