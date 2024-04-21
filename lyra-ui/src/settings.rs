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
  window_x: FieldData<WindowCoordinate>,
  window_y: FieldData<WindowCoordinate>,
  webq_label: FieldData<String>,
  webq_template: FieldData<Template>,
  webq_image: FieldData<String>,
}

impl TryParse for Template {
    type Output = Template;
    fn try_parse(v: &String) -> Result<Self::Output, String> {
        todo!()
    }
}

impl Validate for Template {
    type Input = Template;
    fn validate(v: &Self::Input) -> Result<(), String> {
        todo!()
    }
}

impl TryParse for String {
  type Output = String;
  fn try_parse(v: &String) -> Result<Self::Output, String> {
    Ok(v.clone())
  }
}

impl Validate for String {
  type Input = String;
  fn validate(_v: &Self::Input) -> Result<(), String> {
    Ok(())
  }
}

#[derive(Clone)]
struct WindowCoordinate(f32);
impl ToString for WindowCoordinate {
  fn to_string(&self) -> String {
    self.0.to_string()
  }
}

// TODO: Is anything derivable here? Like a default?
impl TryParse for WindowCoordinate {
  type Output = Self;
  fn try_parse(v: &String) -> Result<Self::Output, String> {
    v.parse::<f32>()
      .map_err(|_| format!("{} is not a number", v))
      .map(|v| WindowCoordinate(v))
  }
}

impl Validate for WindowCoordinate {
  type Input = Self;
  fn validate(v: &Self::Input) -> Result<(), String> {
    if v.0 < 0.0 {
      return Err("Must be larger than 0".into());
    }
    Ok(())
  }
}

impl LyraSettings {
  pub fn new(config: Arc<Config>) -> Self {
    // TODO: This construction logic might be less redundant if all fields
    // went into a mutable struct initialized with ::default()?
    let window_x: WindowCoordinate;
    let window_y: WindowCoordinate;
    let mut webq_label = String::new();
    let mut webq_template = Template::default();
    let mut webq_image = String::new();
    {
      let cfg = config.get();
      match cfg.styles.window_placement {
        Placement::XY(x, y) => {
          window_x = WindowCoordinate(x);
          window_y = WindowCoordinate(y);
        }
      }
      if let Some(webq) = &cfg.webq.default_searcher {
        webq_label = webq.label.clone();
        webq_template = webq.template.clone();
        webq_image = webq.icon.clone();
      }
    }
    LyraSettings {
      id: ViewportId::from_hash_of(LYRA_SETTINGS),
      title: LYRA_SETTINGS.into(),
      visible: Arc::new(RwLock::new(false)),
      window_x: FieldData::new(window_x),
      window_y: FieldData::new(window_y),
      webq_label: FieldData::new(webq_label),
      webq_template: FieldData::new(webq_template),
      webq_image: FieldData::new(webq_image),
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
            ui.add(
              Input::of("X:", &mut self.window_x)
                .desired_width(35.0),
            );
            ui.add(
              Input::of("Y:", &mut self.window_y)
                .desired_width(35.0),
            );
          });
          ui.separator();
          ui.label("Default Search");
          ui.vertical(|ui| {
            ui.add(Input::of("Label:", &mut self.webq_label).desired_width(200.0));
            ui.add(Input::of("Template:", &mut self.webq_template).desired_width(400.0));
            ui.horizontal(|ui| {
              let img = self
                .webq_image
                .value
                .clone()
                .unwrap_or_else(|_| String::new());
              ui.add(Input::of("Image:", &mut self.webq_image).desired_width(400.0));
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
                inner.styles.window_placement = Placement::XY(res.window_x.0, res.window_y.0);
                inner.webq.default_searcher = Some(WebqSearchConfig {
                  label: res.webq_label,
                  shortname: "".into(),
                  template: res.webq_template,
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
// TODO: Should parse & validate just be traits T should satisfy? Can newtype when the behavior is diff.
//       That would also make the Input constructor simplified no wouldn't it :thinkies:
struct FieldData<T: Clone + TryParse<Output = T> + Validate<Input = T>> {
  buffer: String,
  value: Result<T, String>,
}

trait TryParse {
  type Output;
  fn try_parse(v: &String) -> Result<Self::Output, String>;
}

trait Validate {
  type Input;
  fn validate(v: &Self::Input) -> Result<(), String>;
}

impl<T: ToString + Clone + TryParse<Output = T> + Validate<Input = T>> FieldData<T> {
  pub fn new(value: T) -> FieldData<T> {
    let buffer = value.to_string();
    FieldData {
      value: Ok(value),
      buffer,
    }
  }

  pub fn parse_and_validate(&mut self) {
    self.value = <T as TryParse>::try_parse(&self.buffer);
    if let Ok(v) = self.value.as_ref() {
      if let Err(err) = <T as Validate>::validate(v) {
        self.value = Err(err);
      }
    }
  }
}

struct Input<'a, T: Clone + TryParse<Output = T> + Validate<Input = T>> {
  label: &'a str,
  field: &'a mut FieldData<T>,
  desired_width: Option<f32>,
}

impl<'a, T: Clone + TryParse<Output = T> + Validate<Input = T>> Input<'a, T> {
  pub fn of(label: &'a str, field: &'a mut FieldData<T>) -> Input<'a, T> {
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

impl<'a, T: ToString + Clone + TryParse<Output = T> + Validate<Input = T>> Widget for Input<'a, T> {
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
