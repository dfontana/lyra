#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::{
  window::WindowBuilder, ActivationPolicy, AppHandle, CustomMenuItem, GlobalShortcutManager,
  Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, Window, WindowEvent, WindowUrl,
};

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
      WindowEvent::Focused(focused) => {
        // hide window whenever it loses focus
        if !focused {
          println!("hiding!");
          event.window().hide().unwrap();
        }
      }
      _ => {}
    })
    .setup(|app| {
      #[cfg(target_os = "macos")]
      app.set_activation_policy(ActivationPolicy::Accessory);

      let handle = app.handle();
      app
        .global_shortcut_manager()
        .register("CmdOrCtrl+Space", move || {
          let win = handle
            .get_window("lyra-main")
            .expect("Framework should have built");
          let is_updated = match win.is_visible() {
            Ok(true) => win.hide(),
            Ok(false) => win.set_focus(),
            Err(err) => Err(err),
          };
          if let Err(err) = is_updated {
            println!("Failed to toggle window: {}", err);
          }
        })?;
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
