use serde_json::json;
use tauri::Window;
use tracing::info;

pub fn close_win(window: &Window) {
  if let Err(err) = window.hide() {
    info!("Failed to close window: {}", err);
    return;
  }
  if let Err(err) = window.emit("reset", json!({"reset": true})) {
    info!("Failed to reset state: {}", err);
    return;
  }
}

#[tauri::command]
pub fn close(window: tauri::Window) -> Result<(), String> {
  close_win(&window);
  Ok(())
}
