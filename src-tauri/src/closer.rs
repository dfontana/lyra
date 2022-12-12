use std::sync::Arc;

use serde_json::json;
use tauri::{LogicalSize, Size, Window};
use tracing::{error, info};

use crate::config::{Config, Styles};

pub fn resize_to(window: &Window, config: Arc<Config>, height: usize) -> Result<(), String> {
  let Styles {
    option_height,
    option_width,
    ..
  } = config.get().styles;
  window
    .set_size(Size::Logical(LogicalSize {
      width: option_width,
      height: option_height * height as f64,
    }))
    .map_err(|e| {
      error!("Failed to resize window {}", e);
      "Failed to resize window".into()
    })
}

pub fn reset_size_impl(window: &Window, config: Arc<Config>) -> Result<(), String> {
  resize_to(window, config, 1)
}

#[tauri::command]
pub fn reset_size(
  window: tauri::Window,
  config: tauri::State<'_, Arc<Config>>,
) -> Result<(), String> {
  reset_size_impl(&window, (*config).clone())
}

pub fn close_win(window: &Window) {
  if let Err(err) = window.hide() {
    info!("Failed to close window: {}", err);
    return;
  }
  if let Err(err) = window.emit("reset", json!({"reset": true})) {
    info!("Failed to reset state: {}", err);
  }
}

#[tauri::command]
pub fn close(window: tauri::Window) -> Result<(), String> {
  close_win(&window);
  Ok(())
}
