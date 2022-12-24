#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
mod calc;
mod closer;
mod config;
mod convert;
mod launcher;
mod lookup;
mod page;

use std::{error::Error, io::Read, path::Path, sync::Arc};

use config::{Config, Placement, Styles};
use launcher::Launcher;
use lookup::applookup::AppLookup;
use page::{MainData, Page, SettingsData};
use tauri::{
  http::{Request, Response, ResponseBuilder},
  ActivationPolicy, App, AppHandle, CustomMenuItem, GlobalShortcutManager, Manager, Menu,
  MenuEntry, MenuItem, Submenu, SystemTray, SystemTrayEvent, SystemTrayMenu, Window, WindowEvent,
};
use tracing::{error, info};

fn open_settings(page: Page, app: &AppHandle) -> Result<(), anyhow::Error> {
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

fn open_app(page: Page, app: &App, cfg: Arc<Config>) -> Result<(), anyhow::Error> {
  let Styles {
    window_placement, ..
  } = cfg.get().styles;
  let mut win = Window::builder(app, page.id(), tauri::WindowUrl::App("index.html".into()))
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
        }
        Err(err) => {
          info!("Failed to toggle window: {}", err);
        }
      }
    })?;
  Ok(())
}

fn handle_style(
  cfg: &Arc<Config>,
  app: &AppHandle,
  request: &Request,
) -> Result<Response, Box<dyn Error>> {
  // Ensures we have default styles initialized. Since we need a path
  // resolver to find the resources dir with the defaults, we can't
  // do this during config object setup
  cfg.init_styles(
    app.path_resolver().resource_dir().unwrap(),
    cfg!(debug_assertions),
  )?;

  info!("Requested Stylesheet: {}", request.uri());
  let path = cfg.get_app_styles_path()?.join(
    request
      .uri()
      .strip_prefix("styles://")
      .and_then(|s| s.strip_suffix('/'))
      .map(Path::new)
      .unwrap(),
  );
  info!("Parsed: {:?}", path);
  let mut content = std::fs::File::open(path)?;
  let mut buf = Vec::new();
  content.read_to_end(&mut buf)?;
  info!("Sent stylesheet");
  ResponseBuilder::new()
    .mimetype("text/css")
    .status(200)
    .body(buf)
}

fn main() {
  if let Err(err) = config::init_logs() {
    error!("Failed to start logger: {}", err);
    return;
  }

  let config = match Config::get_or_init_config() {
    Ok(c) => Arc::new(c),
    Err(err) => {
      info!("Failed to initialize config: {}", err);
      return;
    }
  };
  let global_cfg = config.clone();
  let style_cfg = config.clone();
  let apps = AppLookup::new(config.clone());
  if let Err(e) = apps.init() {
    info!("Failed to initialize app icons: {}", e);
    return;
  }

  tauri::Builder::default()
    .system_tray(
      SystemTray::new().with_menu(
        SystemTrayMenu::new()
          .add_item(CustomMenuItem::new("settings".to_string(), "Settings"))
          .add_item(CustomMenuItem::new("quit".to_string(), "Quit")),
      ),
    )
    .on_system_tray_event(|app, event| {
      if let SystemTrayEvent::MenuItemClick { id, .. } = event {
        match id.as_str() {
          "quit" => {
            std::process::exit(0);
          }
          "settings" => {
            let page = Page::Settings(SettingsData::builder().build().unwrap());
            if let Err(err) = open_settings(page, app) {
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
      let page = Page::Main(MainData::builder().build()?);
      open_app(page, app, global_cfg)?;
      Ok(())
    })
    .manage(config.clone())
    .manage(Launcher::new(config, apps))
    .invoke_handler(tauri::generate_handler![
      calc::calculate,
      closer::close,
      convert::image_data_url,
      config::get_config,
      config::save_bookmarks,
      config::save_engine,
      config::save_searchers,
      config::validate_template,
      launcher::submit,
      launcher::search,
    ])
    .register_uri_scheme_protocol("styles", move |app, request| {
      handle_style(&style_cfg, app, request)
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
