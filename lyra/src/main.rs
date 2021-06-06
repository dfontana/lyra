extern crate clap;
#[macro_use]
extern crate include_dir;
extern crate tokio;

mod error;
mod event;
mod hotkeys;
mod window;

use clap::App;
use event::Event as UserEvent;

use wry::application::{
  event::{Event, StartCause, WindowEvent},
  event_loop::ControlFlow,
};

#[tokio::main]
async fn main() {
  let _matches = App::new("Lyra").get_matches();

  let (evloop, webview) = window::configure().expect("Window Setup Failed");

  hotkeys::launch(evloop.create_proxy());

  evloop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;

    // TODO: Tao can support hotkeys via this loop, but it doesn't appear to be
    //       working at the device level just yet. They are working on integrating
    //       as of writing; but for now can manually use tauri-hotkey
    //       https://github.com/tauri-apps/tao/issues/33
    match event {
      Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
      Event::UserEvent(ev) => match ev {
        UserEvent::Show => {
          webview.window().set_visible(true);
          webview.window().set_focus();
        }
        UserEvent::Hide => {
          webview.window().set_visible(false);
        }
        _ => (),
      },
      Event::WindowEvent {event: WindowEvent::Focused(false), ..} => {
        webview.window().set_visible(false);
      }
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => *control_flow = ControlFlow::Exit,
      _ => (),
    }
  });
}
