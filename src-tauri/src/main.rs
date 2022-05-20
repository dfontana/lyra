#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
mod closer;
mod config;
mod launcher;
mod page;

use anyhow::anyhow;
use closer::Closer;
use config::{Bookmark, Config, InnerConfig};
use launcher::{Launcher, SearchOption};
use page::{MainData, Page, SettingsData};
use reqwest::header::CONTENT_TYPE;
use tauri::{
  ActivationPolicy, AppHandle, CustomMenuItem, GlobalShortcutManager, Manager, Menu, MenuEntry,
  MenuItem, Submenu, SystemTray, SystemTrayEvent, SystemTrayMenu, Window, WindowEvent,
};
use tracing::{error, info};

#[tauri::command]
fn close(window: tauri::Window) -> Result<(), String> {
  Closer::close(&window);
  Ok(())
}

#[tauri::command]
async fn image_data_url(url: String) -> Result<String, String> {
  _convert_image(url).await.map_err(|err| {
    error!("Failed to parse image to data-url: {}", err);
    "Could not convert image to data-url".into()
  })
}

async fn _convert_image(url: String) -> Result<String, anyhow::Error> {
  let resp = reqwest::get(url).await?;
  let ctype = match resp.headers().get(CONTENT_TYPE) {
    Some(v) => v.to_str()?,
    None => return Err(anyhow!("Unknown content type")),
  };
  let ctype = match ctype {
    "image/svg+xml" | "image/png" | "image/vnd.microsoft.icon" | "image/jpeg" => ctype.to_owned(),
    _ => return Err(anyhow!("Unsupported Content Type: {}", ctype)),
  };
  let body = resp.bytes().await?;
  let str = format!("data:{};base64,{}", ctype, base64::encode(&body));
  info!("Found: {}", str);
  Ok(str)
}

#[tauri::command]
fn get_config(config: tauri::State<Config>) -> InnerConfig {
  (*config.config.lock().unwrap()).clone()
}

#[tauri::command]
fn save_bookmarks(config: tauri::State<Config>, bookmarks: Vec<Bookmark>) -> Result<(), String> {
  config.update_bookmarks(bookmarks).map_err(|err| {
    error!("Failed to save bookmarks: {}", err);
    "Failed to save bookmarks".into()
  })
}

#[tauri::command]
async fn search(
  launcher: tauri::State<'_, Launcher>,
  search: String,
) -> Result<Vec<SearchOption>, String> {
  Ok(launcher.get_options(&search).await)
}

#[tauri::command]
fn submit(
  launcher: tauri::State<Launcher>,
  selection: usize,
  window: tauri::Window,
) -> Result<(), String> {
  match launcher.launch(selection) {
    Ok(()) => {
      Closer::close(&window);
      Ok(())
    }
    Err(err) => {
      info!("Failed to launch option {}", err);
      Err("Failed to launch".into())
    }
  }
}

fn open_settings(app: &AppHandle) -> Result<(), anyhow::Error> {
  let page = Page::Settings(SettingsData::builder().build()?);
  if let Some(win) = app.get_window(&page.id()) {
    win.show()?;
    win.set_focus()?;
    return Ok(());
  }
  Window::builder(app, page.id(), tauri::WindowUrl::App("index.html".into()))
    .center()
    .title("Lyra Settings")
    .focus()
    .menu(Menu::with_items([
      #[cfg(target_os = "macos")]
      MenuEntry::Submenu(Submenu::new(
        "Edit",
        Menu::with_items([
          MenuItem::Undo.into(),
          MenuItem::Redo.into(),
          MenuItem::Cut.into(),
          MenuItem::Copy.into(),
          MenuItem::Paste.into(),
          MenuItem::SelectAll.into(),
        ]),
      )),
    ]))
    .initialization_script(&page.init_script()?)
    .build()?;
  Ok(())
}

fn main() {
  if let Err(err) = config::init_logs() {
    error!("Failed to start logger: {}", err);
    return;
  }

  let config = match Config::get_or_init_config() {
    Ok(c) => c,
    Err(err) => {
      info!("Failed to initialize config: {}", err);
      return;
    }
  };

  let tray_menu = SystemTrayMenu::new()
    .add_item(CustomMenuItem::new("settings".to_string(), "Settings"))
    .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));

  tauri::Builder::default()
    .system_tray(SystemTray::new().with_menu(tray_menu))
    .on_system_tray_event(|app, event| match event {
      SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
        "quit" => {
          std::process::exit(0);
        }
        "settings" => {
          if let Err(err) = open_settings(app) {
            error!("Failed to open settings: {}", err);
          }
        }
        _ => {}
      },
      _ => {}
    })
    .on_window_event(|event| match event.event() {
      WindowEvent::Focused(focused) => {
        if !focused && event.window().label() == Page::Main(MainData::default()).id() {
          // Closer::close(&event.window());
        }
      }
      _ => {}
    })
    .setup(|app| {
      #[cfg(target_os = "macos")]
      app.set_activation_policy(ActivationPolicy::Accessory);

      // TODO code assumes input is 38px large, and each result is 18px with max of 10 results shown.
      let page = Page::Main(
        MainData::builder()
          .style(("OPTION_HEIGHT".into(), 38.into()))
          .style(("INPUT_HEIGHT".into(), 38.into()))
          .style(("FONT_SIZE".into(), 16.into()))
          .build()?,
      );

      Window::builder(app, page.id(), tauri::WindowUrl::App("index.html".into()))
        .inner_size(600f64, 218f64)
        .resizable(false)
        .always_on_top(true)
        .decorations(false)
        .visible(false)
        .fullscreen(false)
        .skip_taskbar(true)
        .center()
        .initialization_script(&page.init_script()?)
        .build()?;

      let handle = app.handle();
      app
        .global_shortcut_manager()
        .register("CmdOrCtrl+Space", move || {
          let win = handle
            .get_window(page.id())
            .expect("Framework should have built");
          let is_updated = match win.is_visible() {
            Ok(true) => Ok(Closer::close(&win)),
            Ok(false) => win.set_focus(),
            Err(err) => Err(err),
          };
          if let Err(err) = is_updated {
            info!("Failed to toggle window: {}", err);
          }
        })?;
      Ok(())
    })
    .manage(config.clone())
    .manage(Launcher::new(config))
    .invoke_handler(tauri::generate_handler![
      close,
      get_config,
      image_data_url,
      save_bookmarks,
      submit,
      search
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
