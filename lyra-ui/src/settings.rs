use derive_more::{Display, FromStr};
use egui::{Color32, Layout, Stroke, TextEdit, Vec2, ViewportId, Widget};
use egui_extras::{Column, TableBuilder};
use form::{FormField, FormFieldData, FormResult, TryParse, Validate};
use global_hotkey::hotkey::HotKey;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::warn;

use crate::{
  config::{Config, Placement, WebqSearchConfig},
  icon_ui::{data_or_url, Icon},
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
struct LyraWebqForm {
  label: FormField<WebqLabel>,
  shortname: FormField<WebqShortname>,
  template: FormField<Template>,
  image: FormField<WebqImage>,
  index: Option<usize>,
}
impl LyraWebqForm {
  fn clear(&mut self) {
    self.label = FormField::default();
    self.shortname = FormField::default();
    self.template = FormField::default();
    self.image = FormField::default();
    self.index = None;
  }
}

#[derive(FormResult, Default)]
struct LyraSettingsForm {
  window_x: FormField<WindowCoordinate>,
  window_y: FormField<WindowCoordinate>,
  // Globals
  hotkey: FormField<FormHotKey>,
  // Default searcher
  webq_label: FormField<WebqLabel>,
  webq_template: FormField<Template>,
  webq_image: FormField<WebqImage>,
  // All other searchers & searcher form
  searcher_form: LyraWebqForm,
  webq_searchers: Vec<WebqSearchConfig>,
}

impl Validate for Template {
  fn validate(v: &Self) -> Result<(), String> {
    match (*v).trim().is_empty() {
      true => Err("Cannot be blank".into()),
      false => Ok(()),
    }
  }
}

#[derive(Clone, Default, Display, FromStr, FormFieldData)]
struct FormHotKey(String);
impl Validate for FormHotKey {
  fn validate(v: &Self) -> Result<(), String> {
    v.0
      .parse::<HotKey>()
      .map(|_| ())
      .map_err(|e| format!("{}", e))
  }
}

#[derive(Clone, Default, Display, FromStr, FormFieldData)]
struct WebqShortname(String);
impl Validate for WebqShortname {
  fn validate(v: &Self) -> Result<(), String> {
    match v.0.trim().is_empty() {
      true => Err("Cannot be blank".into()),
      false => Ok(()),
    }
  }
}

#[derive(Clone, Default, Display, FromStr, FormFieldData)]
struct WebqLabel(String);
impl Validate for WebqLabel {
  fn validate(v: &Self) -> Result<(), String> {
    match v.0.trim().is_empty() {
      true => Err("Cannot be blank".into()),
      false => Ok(()),
    }
  }
}

#[derive(Clone, Default, Display, Validate, FormFieldData)]
struct WebqImage(String);
impl TryParse for WebqImage {
  fn try_parse(v: &String) -> Result<Self, String> {
    data_or_url(v)
      .map(|v| WebqImage(v))
      .map_err(|e| e.to_string())
  }
}

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
      form.webq_searchers = cfg.webq.searchers.values().map(|w| w.clone()).collect();
      form.hotkey = FormField::new(FormHotKey(cfg.hotkey.parse().unwrap()));
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
    // TODO: When the Setting UI is closed it shoudl reset the data inside it
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
          ui.horizontal_top(|ui| {
            ui.add(Input::of("Hotkey:", &mut self.form.hotkey).desired_width(200.0));
          });
          ui.separator();
          ui.vertical(|ui| {
            ui.label("Default Search");
            ui.add(Input::of("Label:", &mut self.form.webq_label).desired_width(200.0));
            ui.add(Input::of("Template:", &mut self.form.webq_template).desired_width(400.0));
            ui.horizontal(|ui| {
              let mb_img = &self.form.webq_image.value;
              let mb_lbl = &self.form.webq_label.value;
              let mb_ico = mb_img.clone().and_then(|img| {
                mb_lbl.clone().and_then(|lbl| {
                  Icon::try_from((img.0.as_str(), lbl.0.as_str())).map_err(|e| e.to_string())
                })
              });
              ui.add(Input::of("Image:", &mut self.form.webq_image).desired_width(400.0));
              if let Ok(ico) = mb_ico {
                ico.render(ui);
              }
            });
          });
          ui.separator();
          ui.label("Bookmarks");
          ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
              Vec2::new(450.0, 200.0),
              Layout::top_down(egui::Align::Min),
              |ui| {
                ui.add(Input::of("Label:", &mut self.form.searcher_form.label));
                ui.add(Input::of(
                  "Shortname:",
                  &mut self.form.searcher_form.shortname,
                ));
                ui.add(Input::of(
                  "Template:",
                  &mut self.form.searcher_form.template,
                ));
                ui.add(
                  Input::of("Image:", &mut self.form.searcher_form.image).desired_width(400.0),
                );
              },
            );
            let mb_img = &self.form.searcher_form.image.value;
            let mb_lbl = &self.form.searcher_form.label.value;
            let mb_ico = mb_img.clone().and_then(|img| {
              mb_lbl.clone().and_then(|lbl| {
                Icon::try_from((img.0.as_str(), lbl.0.as_str())).map_err(|e| e.to_string())
              })
            });
            if let Ok(ico) = mb_ico {
              ico.render(ui);
            }
          });
          ui.horizontal(|ui| {
            let text = if self.form.searcher_form.index == None {
              "Add bookmark"
            } else {
              "Update bookmark"
            };
            if ui.button(text).clicked() {
              let idx = self.form.searcher_form.index;
              if let Ok(res) = TryInto::<LyraWebqFormFormResult>::try_into(&self.form.searcher_form)
              {
                let cfg = WebqSearchConfig {
                  label: res.label.0,
                  shortname: res.shortname.0,
                  template: res.template,
                  icon: res.image.0,
                };
                if let Some(id) = idx {
                  self.form.webq_searchers.remove(id);
                  self.form.webq_searchers.insert(id, cfg);
                } else {
                  self.form.webq_searchers.push(cfg);
                }
                self.form.searcher_form.clear();
              }
            }
            if ui.button("Clear").clicked() {
              self.form.searcher_form.clear();
            }
          });

          ui.vertical(|ui| {
            TableBuilder::new(ui)
              .striped(true)
              .resizable(false)
              .vscroll(true)
              .auto_shrink([false, true])
              .column(Column::exact(100.0))
              .column(Column::exact(100.0))
              .column(Column::exact(210.0))
              .column(Column::exact(50.0))
              .column(Column::exact(35.0))
              .column(Column::exact(50.0))
              .header(18.0, |mut r| {
                for label in ["Label", "Shortname", "Template"] {
                  r.col(|ui| {
                    ui.horizontal_centered(|ui| {
                      ui.label(label);
                    });
                  });
                }
                for label in ["Image", "Edit", "Delete"] {
                  r.col(|ui| {
                    ui.vertical_centered(|ui| {
                      ui.label(label);
                    });
                  });
                }
              })
              .body(|mut body| {
                body
                  .ui_mut()
                  .style_mut()
                  .visuals
                  .widgets
                  .noninteractive
                  .bg_stroke = Stroke::new(2.0, Color32::RED);
                let rows = self.form.webq_searchers.len();
                body.rows(18.0, rows, |mut row| {
                  let idx = row.index();
                  let data = self.form.webq_searchers.get(idx).unwrap();
                  row.col(|ui| {
                    ui.horizontal_centered(|ui| {
                      ui.label(&data.label);
                    });
                  });
                  row.col(|ui| {
                    ui.horizontal_centered(|ui| {
                      ui.label(&data.shortname);
                    });
                  });
                  row.col(|ui| {
                    ui.horizontal_centered(|ui| {
                      ui.label(&data.template.to_string());
                    });
                  });
                  row.col(|ui| {
                    let mbico = Icon::try_from((data.icon.as_str(), data.label.as_str()))
                      .map_err(|e| e.to_string());
                    if let Ok(ico) = mbico {
                      ui.horizontal_centered(|ui| {
                        ico.render(ui);
                      });
                    }
                  });
                  row.col(|ui| {
                    ui.vertical_centered(|ui| {
                      if ui.button("Edit").clicked() {
                        self.form.searcher_form.label =
                          FormField::new(WebqLabel(data.label.clone()));
                        self.form.searcher_form.shortname =
                          FormField::new(WebqShortname(data.shortname.clone()));
                        self.form.searcher_form.template = FormField::new(data.template.clone());
                        self.form.searcher_form.image =
                          FormField::new(WebqImage(data.icon.clone()));
                        self.form.searcher_form.index = Some(idx);
                      }
                    });
                  });
                  row.col(|ui| {
                    ui.vertical_centered(|ui| {
                      if ui
                        .add_enabled(
                          self.form.searcher_form.index == None,
                          egui::Button::new("Delete"),
                        )
                        .clicked()
                      {
                        self.form.webq_searchers.remove(idx);
                      }
                    });
                  });
                });
              });
          });
          ui.separator();
          if ui.button("Save").clicked() {
            // TODO: Blank forms don't have their fields parse_and_validate
            //       so they all start as valid in their initial state. You
            //       should do that...
            match TryInto::<LyraSettingsFormFormResult>::try_into(&self.form) {
              Ok(res) => {
                let searchers = self
                  .form
                  .webq_searchers
                  .iter()
                  .map(|s| (s.label.clone(), s.to_owned()))
                  .collect();
                self.config.update(move |mut inner| {
                  inner.styles.window_placement = Placement::XY(res.window_x.0, res.window_y.0);
                  inner.webq.default_searcher = Some(WebqSearchConfig {
                    label: res.webq_label.0,
                    shortname: "".into(),
                    template: res.webq_template,
                    icon: res.webq_image.0,
                  });
                  inner.webq.searchers = searchers;
                  // TODO: Add more fields
                  // inner.apps.app_paths;
                  // inner.apps.app_extension;
                  // inner.calc.prefix;
                  // Top level:
                  //  result_count
                  //  styles
                })
              }
              Err(err) => {
                ui.colored_label(Color32::RED, format!("{}", err));
              }
            }
            if let Err(err) = self.config.persist() {
              warn!("Failed to save config update: {}", err);
              ui.colored_label(Color32::RED, format!("{}", err));
            } else {
              ui.colored_label(Color32::GREEN, "Saved!");
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
