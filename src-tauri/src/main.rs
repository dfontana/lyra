#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use serde_json::{json, Value};
use tauri::{
  ActivationPolicy, CustomMenuItem, GlobalShortcutManager, Manager, SystemTray, SystemTrayEvent,
  SystemTrayMenu, Window, WindowEvent,
};

// TODO add trace log

#[tauri::command]
fn close_window(window: tauri::Window) -> Result<(), String> {
  close(&window);
  Ok(())
}

#[tauri::command]
async fn search(search: String) -> Result<Value, String> {
  // TODO stronger type this with Vec, Struct, & derive serde::Serialize
  Ok(json!([{
    "id": 0,
    "value": "First Result"
  },{
    "id": 1,
    "value": "Second Result"
  }]))
}

#[tauri::command]
fn submit(selection: usize, window: tauri::Window) -> Result<(), String> {
  close(&window);
  Ok(())
}

fn close(window: &Window) {
  if let Err(err) = window.hide() {
    println!("Failed to close window: {}", err);
    return;
  }
  if let Err(err) = window.emit("reset", json!({"reset": true})) {
    println!("Failed to reset state: {}", err);
    return;
  }
}

fn main() {
  let tray_menu = SystemTrayMenu::new().add_item(CustomMenuItem::new("quit".to_string(), "Quit"));

  tauri::Builder::default()
    .system_tray(SystemTray::new().with_menu(tray_menu))
    .on_system_tray_event(|_, event| match event {
      SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
        "quit" => {
          std::process::exit(0);
        }
        _ => {}
      },
      _ => {}
    })
    .on_window_event(|event| match event.event() {
      WindowEvent::Focused(focused) if !focused => {
        // close(&event.window());
      }
      _ => {}
    })
    .setup(|app| {
      #[cfg(target_os = "macos")]
      app.set_activation_policy(ActivationPolicy::Accessory);

      // TODO code assumes input is 38px large, and each result is 18px with max of 10 results shown.
      //      Should allow this to be configurable
      let data = json!({
        "calls": {
          "SEARCH": "search",
          "SUBMIT": "submit",
          "CLOSE": "close_window",
        },
        "events": {
          "RESET": "reset",
        },
        "styles": {
          "OPTION_HEIGHT": 18,
          "INPUT_HEIGHT": 38,
          "FONT_SIZE": 16,
        }
      });

      Window::builder(app, "lyra-main", tauri::WindowUrl::App("index.html".into()))
        .inner_size(600f64, 218f64)
        .resizable(false)
        .always_on_top(true)
        .decorations(false)
        .visible(false)
        .fullscreen(false)
        .skip_taskbar(true)
        .center()
        .initialization_script(&format!("window.__LYRA__ = {}", data))
        .build()?;

      let handle = app.handle();
      app
        .global_shortcut_manager()
        .register("CmdOrCtrl+Space", move || {
          let win = handle
            .get_window("lyra-main")
            .expect("Framework should have built");
          let is_updated = match win.is_visible() {
            Ok(true) => Ok(close(&win)),
            Ok(false) => win.set_focus(),
            Err(err) => Err(err),
          };
          if let Err(err) = is_updated {
            println!("Failed to toggle window: {}", err);
          }
        })?;
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![close_window, submit, search])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
