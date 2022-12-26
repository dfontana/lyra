#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
mod calc;
mod closer;
mod config;
mod convert;
mod launcher;
mod page;

use std::sync::Arc;

use config::{Config, Placement, Styles};
use launcher::Launcher;
use page::{MainData, Page, SettingsData};
use tauri::{
  ActivationPolicy, AppHandle, CustomMenuItem, GlobalShortcutManager, Manager, Menu, MenuEntry,
  MenuItem, Submenu, SystemTray, SystemTrayEvent, SystemTrayMenu, Window, WindowEvent,
};
use tracing::{error, info};

fn open_settings(app: &AppHandle) -> Result<(), anyhow::Error> {
  let page = Page::Settings(SettingsData::builder().build()?);
  if let Some(win) = app.get_window(page.id()) {
    win.show()?;
    win.set_focus()?;
    return Ok(());
  }
  Window::builder(app, page.id(), tauri::WindowUrl::App("index.html".into()))
    .center()
    .title("Lyra Settings")
    .focused(true)
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

  let (config, plugins) = match Config::get_or_init_config() {
    Ok((c, p)) => (Arc::new(c), p),
    Err(err) => {
      info!("Failed to initialize config: {}", err);
      return;
    }
  };
  let global_cfg = config.clone();

  let tray_menu = SystemTrayMenu::new()
    .add_item(CustomMenuItem::new("settings".to_string(), "Settings"))
    .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));

  tauri::Builder::default()
    .system_tray(SystemTray::new().with_menu(tray_menu))
    .on_system_tray_event(|app, event| {
      if let SystemTrayEvent::MenuItemClick { id, .. } = event {
        match id.as_str() {
          "quit" => {
            std::process::exit(0);
          }
          "settings" => {
            if let Err(err) = open_settings(app) {
              error!("Failed to open settings: {}", err);
            }
          }
          _ => {}
        }
      }
    })
    .on_window_event(|event| {
      if let WindowEvent::Focused(focused) = event.event() {
        if !focused && event.window().label() == Page::Main(MainData::default()).id() {
          #[cfg(not(debug_assertions))]
          closer::close_win(event.window());
        }
      }
    })
    .setup(move |app| {
      #[cfg(target_os = "macos")]
      app.set_activation_policy(ActivationPolicy::Accessory);

      let Styles {
        option_width,
        option_height,
        font_size,
        window_placement,
        ..
      } = global_cfg.get().styles;
      let page = Page::Main(
        MainData::builder()
          .style(("OPTION_HEIGHT".into(), option_height.into()))
          .style(("INPUT_HEIGHT".into(), option_height.into()))
          .style(("FONT_SIZE".into(), font_size.into()))
          .build()?,
      );

      let mut win = Window::builder(app, page.id(), tauri::WindowUrl::App("index.html".into()))
        .inner_size(option_width, option_height)
        .resizable(false)
        .always_on_top(true)
        .decorations(false)
        .visible(false)
        .fullscreen(false)
        .skip_taskbar(true);
      match window_placement {
        Placement::Center => {
          win = win.center();
        }
        Placement::XY(x, y) => {
          win = win.position(x, y);
        }
      }
      win
        .transparent(true)
        .initialization_script(&page.init_script()?)
        .build()?;

      let handle = app.handle();
      app
        .global_shortcut_manager()
        // TODO: move this into the config so folks can customize the trigger
        .register("CmdOrCtrl+Space", move || {
          let win = handle
            .get_window(page.id())
            .expect("Framework should have built");
          match win.is_visible() {
            Ok(true) => {
              closer::close_win(&win);
            }
            Ok(false) => {
              if let Err(err) = win.set_focus() {
                info!("Failed to toggle window: {}", err)
              }
              if let Err(err) = closer::reset_size_impl(&win, global_cfg.clone()) {
                info!("Failed to toggle window: {}", err)
              }
            }
            Err(err) => {
              info!("Failed to toggle window: {}", err);
            }
          }
        })?;
      Ok(())
    })
    .manage(config.clone())
    .manage(Launcher::new(config, plugins))
    .invoke_handler(tauri::generate_handler![
      calc::calculate,
      closer::close,
      closer::reset_size,
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
