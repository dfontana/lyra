use std::str::FromStr;

use super::{template::Template, Bookmark, Config, InnerConfig, Searcher};
use tracing::error;

#[tauri::command]
pub fn get_config(config: tauri::State<Config>) -> InnerConfig {
  config.get().clone()
}

#[tauri::command]
pub fn save_bookmarks(config: tauri::State<Config>, updates: Vec<Bookmark>) -> Result<(), String> {
  config.update_bookmarks(updates).map_err(|err| {
    error!("Failed to save bookmarks: {}", err);
    "Failed to save bookmarks".into()
  })
}

#[tauri::command]
pub fn save_searchers(config: tauri::State<Config>, updates: Vec<Searcher>) -> Result<(), String> {
  config.update_searchers(updates).map_err(|err| {
    error!("Failed to save searchers: {}", err);
    "Failed to save searchers".into()
  })
}

#[tauri::command]
pub fn validate_template(input: String) -> Result<(), String> {
  match Template::from_str(&input) {
    Ok(_) => Ok(()),
    Err(e) => Err(format!("{}", e)),
  }
}
