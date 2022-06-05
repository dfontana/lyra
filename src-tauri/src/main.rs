#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
mod closer;
mod config;
mod convert;
mod launcher;
mod lookup;
mod page;

use config::{Config, Styles};
use launcher::Launcher;
use lookup::applookup::AppLookup;
use page::{MainData, Page, SettingsData};
use tauri::{
  ActivationPolicy, AppHandle, CustomMenuItem, GlobalShortcutManager, Manager, Menu, MenuEntry,
  MenuItem, Submenu, SystemTray, SystemTrayEvent, SystemTrayMenu, Window, WindowEvent,
};
use tracing::{error, info};

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
  let global_cfg = config.clone();
  let apps = AppLookup::new(global_cfg.clone());

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
          #[cfg(not(debug_assertions))]
          Closer::close(&event.window());
        }
      }
      _ => {}
    })
    .setup(move |app| {
      #[cfg(target_os = "macos")]
      app.set_activation_policy(ActivationPolicy::Accessory);

      let Styles {
        option_width,
        option_height,
        font_size,
        ..
      } = config.get().styles;
      let page = Page::Main(
        MainData::builder()
          .style(("OPTION_HEIGHT".into(), option_height.into()))
          .style(("INPUT_HEIGHT".into(), option_height.into()))
          .style(("FONT_SIZE".into(), font_size.into()))
          .build()?,
      );

      Window::builder(app, page.id(), tauri::WindowUrl::App("index.html".into()))
        .inner_size(option_width, option_height)
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
            Ok(true) => Ok(closer::close_win(&win)),
            Ok(false) => win.set_focus(),
            Err(err) => Err(err),
          };
          if let Err(err) = is_updated {
            info!("Failed to toggle window: {}", err);
          }
        })?;
      Ok(())
    })
    .manage(global_cfg.clone())
    .manage(Launcher::new(global_cfg, apps))
    .invoke_handler(tauri::generate_handler![
      closer::close,
      convert::image_data_url,
      config::get_config,
      config::save_bookmarks,
      config::save_engine,
      config::save_searchers,
      config::validate_template,
      launcher::submit,
      launcher::search,
      launcher::select_searcher
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
