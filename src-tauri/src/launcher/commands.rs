use tauri::{LogicalSize, Size};
use tracing::{error, info};

use crate::{
  closer,
  config::{Config, Styles},
};

use super::{Launcher, SearchOption};

fn resize(
  height: usize,
  window: tauri::Window,
  config: tauri::State<'_, Config>,
) -> Result<(), String> {
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

#[tauri::command]
pub async fn search(
  window: tauri::Window,
  launcher: tauri::State<'_, Launcher>,
  config: tauri::State<'_, Config>,
  search: String,
) -> Result<Vec<SearchOption>, String> {
  let options = launcher.get_options(&search).await;
  resize(options.len() + 1, window, config)?;
  Ok(options)
}

#[tauri::command]
pub fn select_searcher(
  config: tauri::State<'_, Config>,
  window: tauri::Window,
) -> Result<(), String> {
  resize(2, window, config)
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
