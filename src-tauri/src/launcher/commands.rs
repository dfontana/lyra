use crate::{closer, config::Config};
use std::sync::Arc;
use tracing::info;

use super::{Launcher, SortWrapper};

#[tauri::command]
pub async fn search(
  window: tauri::Window,
  launcher: tauri::State<'_, Launcher>,
  config: tauri::State<'_, Arc<Config>>,
  search: String,
) -> Result<Vec<SortWrapper>, String> {
  let options = launcher.get_options(&search).await;
  closer::resize_to(&window, (*config).clone(), options.len() + 1)?;
  Ok(options)
}

#[tauri::command]
pub fn select_searcher(
  config: tauri::State<'_, Arc<Config>>,
  window: tauri::Window,
) -> Result<(), String> {
  closer::resize_to(&window, (*config).clone(), 2)
}

#[tauri::command]
pub fn submit(
  launcher: tauri::State<Launcher>,
  selected: SortWrapper,
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
