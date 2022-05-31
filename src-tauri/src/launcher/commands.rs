use tauri::{LogicalSize, Size};
use tracing::{error, info};

use crate::{
  closer,
  config::{Config, Styles},
};

use super::{Launcher, SearchOption};

#[tauri::command]
pub async fn search(
  window: tauri::Window,
  launcher: tauri::State<'_, Launcher>,
  config: tauri::State<'_, Config>,
  search: String,
) -> Result<Vec<SearchOption>, String> {
  let options = launcher.get_options(&search).await;
  let Styles {
    option_height,
    option_width,
    ..
  } = config.get().styles;
  window
    .set_size(Size::Logical(LogicalSize {
      width: option_width,
      height: option_height * (options.len() + 1) as f64,
    }))
    .map_err(|e| {
      error!("Failed to resize window {}", e);
      "Failed to resize window"
    })?;
  Ok(options)
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
