use std::sync::Arc;

use egui::ViewportId;
use egui_extras::{Column, TableBuilder};
use parking_lot::RwLock;

const LYRA_SETTINGS: &str = "Lyra Settings";

pub struct LyraSettings {
  pub id: ViewportId,
  pub title: String,
  pub visible: Arc<RwLock<bool>>,
  // AppData
  window_x: String,
  window_y: String,
  webq_label: String,
  webq_template: String,
  webq_image: String,
}

impl Default for LyraSettings {
  fn default() -> Self {
    LyraSettings {
      id: ViewportId::from_hash_of(LYRA_SETTINGS),
      title: LYRA_SETTINGS.into(),
      visible: Arc::new(RwLock::new(false)),
      window_x: String::new(),
      window_y: String::new(),
      webq_label: String::new(),
      webq_template: String::new(),
      webq_image: String::new(),
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
            ui.text_edit_singleline(&mut self.window_x);
            ui.text_edit_singleline(&mut self.window_y);
          });
          ui.separator();
          ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.webq_label);
            ui.text_edit_singleline(&mut self.webq_template);
            let img = self.webq_image.clone();
            ui.text_edit_singleline(&mut self.webq_image);
            if !img.is_empty() {
              ui.image(img);
            }
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
            // TODO: Save form
          }
        });
      },
    );
  }
}
