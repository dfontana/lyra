use std::sync::Arc;

use egui::ViewportId;
use parking_lot::RwLock;

const LYRA_SETTINGS: &str = "Lyra Settings";

pub struct LyraSettings {
  pub id: ViewportId,
  pub title: String,
  pub visible: Arc<RwLock<bool>>,
}

impl Default for LyraSettings {
  fn default() -> Self {
    LyraSettings {
      id: ViewportId::from_hash_of(LYRA_SETTINGS),
      title: LYRA_SETTINGS.into(),
      visible: Arc::new(RwLock::new(false)),
    }
  }
}

impl LyraSettings {
  pub fn update(&self, ctx: &egui::Context) {
    if !*self.visible.read() {
      return;
    }
    ctx.show_viewport_immediate(
      self.id,
      egui::ViewportBuilder::default()
        .with_title(&self.title)
        .with_inner_size([600.0, 500.0]),
      |ctx, _| {
        egui::CentralPanel::default().show(ctx, |ui| {
          ui.label("Hello from immediate viewport");
        });

        if ctx.input(|i| i.viewport().close_requested()) {
          *self.visible.write() = false;
        }
      },
    );
  }
}
