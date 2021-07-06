extern crate clap;
#[macro_use]
extern crate include_dir;
#[macro_use]
extern crate lazy_static;
extern crate tokio;

mod error;
mod event;
mod window;

use clap::App;

use std::sync::{Arc, Mutex};
use tao::{
  accelerator::{Accelerator, SysMods},
  event::TrayEvent,
  keyboard::KeyCode,
};
use wry::application::{
  event::{Event, StartCause, WindowEvent},
  event_loop::ControlFlow,
  platform::global_shortcut::ShortcutManager,
};

#[cfg(target_os = "windows")]
use wry::application::platform::windows::SystemTrayExtWindows;

lazy_static! {
  static ref IS_VISIBLE: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

#[tokio::main]
async fn main() {
  let _matches = App::new("Lyra").get_matches();

  let (evloop, systemtray, webview) = window::configure().expect("Window Setup Failed");

  let mut hotkeys = ShortcutManager::new(&evloop);
  let toggleopenkey = Accelerator::new(SysMods::Cmd, KeyCode::Space);
  hotkeys.register(toggleopenkey.clone()).unwrap();

  evloop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;

    // Event::UserEvent could be used if evloop.create_proxy() is used to submit an event
    match event {
      Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
      Event::WindowEvent {
        event: WindowEvent::Focused(false),
        ..
      } => {
        webview.window().set_visible(false);
      }
      Event::TrayEvent {
        event: TrayEvent::LeftClick,
        ..
      } => {
        #[cfg(target_os = "windows")]
        systemtray.remove();
        *control_flow = ControlFlow::Exit;
      }
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => *control_flow = ControlFlow::Exit,
      Event::GlobalShortcutEvent(hotkey_id) if hotkey_id == toggleopenkey.clone().id() => {
        let mut vis = IS_VISIBLE.lock().unwrap();
        if !*vis {
          webview.focus();
          webview.window().set_visible(true);
          webview.window().set_focus();
        } else {
          webview.window().set_visible(false);
        }
        *vis = !*vis;
      }
      _ => (),
    }
  });
}
