use crate::{closer, config::Config};
use std::sync::Arc;
use tracing::info;

use super::{Launcher, SearchOption};

#[tauri::command]
pub async fn search(
  window: tauri::Window,
  launcher: tauri::State<'_, Launcher>,
  config: tauri::State<'_, Arc<Config>>,
  search: String,
) -> Result<Vec<SearchOption>, String> {
  Ok(launcher.get_options(&search).await)
}

#[tauri::command]
pub fn submit(
  launcher: tauri::State<Launcher>,
  selected: SearchOption,
  window: tauri::Window,
) -> Result<(), String> {
  match launcher.launch(selected) {
    Ok(()) => {
      closer::close_win(&window);
      Ok(())
    }
    Err(err) => {
      info!("Failed to launch option {}", err);
      Err("Failed to launch".into())
    }
  }
}
