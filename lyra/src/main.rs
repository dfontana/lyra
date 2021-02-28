extern crate clap;
extern crate daemonize;
#[macro_use]
extern crate include_dir;
extern crate tokio;

mod error;
mod protocol;

use clap::{App, Arg};
use keys::{Key, Keyset, Listener};
use std::fs::File;
use tokio::{sync::broadcast, task};
use wry::{Application, Attributes};

use daemonize::Daemonize;

#[tokio::main]
async fn main() {
  let matches = App::new("Lyra")
    .arg(
      Arg::with_name("foreground")
        .short("f")
        .help("Run Lyra in the foreground rather than as a daemon"),
    )
    .get_matches();

  if matches.is_present("foreground") {
    println!("Launching foreground...");
    run().await;
    return;
  }

  println!("Starting Daemon...");
  let stdout = File::create("/tmp/daemon.out").unwrap();
  let stderr = File::create("/tmp/daemon.err").unwrap();

  let daemonize = Daemonize::new()
    .pid_file("/tmp/daemon.pid")
    .stdout(stdout)
    .stderr(stderr)
    .exit_action(|| println!("Executed before master process exits"));

  match daemonize.start() {
    Ok(_) => {
      run().await;
    }
    Err(e) => eprintln!("Error, {}", e),
  }
}

async fn run() {
  let y_offset = 25f64;
  let (disp_w, _) = (1280f64, 800f64);
  let (bar_w, bar_h) = ((disp_w * 0.9f64).floor(), 32f64);
  let (bar_x, bar_y) = (((disp_w - bar_w) / 2f64).floor(), y_offset);

  let prot = protocol::build();

  let mut app = Application::new().expect("it failed to start");
  let attributes = Attributes {
    url: Some("lyra://index.html".to_string()),
    resizable: false,
    visible: false,
    decorations: false,
    transparent: true,
    always_on_top: true,
    width: bar_w,
    height: bar_h,
    x: Some(bar_x),
    y: Some(bar_y),
    skip_taskbar: true,
    ..Default::default()
  };
  let window1 = app
    .add_window_with_configs(attributes, None, Some(prot))
    .expect("It failed");

  let (tx, mut rx) = broadcast::channel(16);
  task::spawn(async move {
    println!("[send] Launching listener");
    Listener::new()
      .add_up(Keyset::new(Key::Space, vec![Key::MetaLeft]))
      .listen(move |e: Keyset| {
        let sender = tx.clone();
        task::spawn(async move {
          match sender.send(e.to_owned()) {
            Err(e) => println!("[send] Failed {:?}", e),
            Ok(_) => println!("[send] Emitted: {}", e),
          }
        });
      })
      .expect("Failed to start listener");
    loop {}
  });

  task::spawn(async move {
    println!("[recv] Launching handler");
    let mut is_visible = false;
    loop {
      match rx.recv().await {
        Err(e) => println!("[recv] Failed {:?}", e),
        Ok(v) => {
          println!("[recv] {}", v);
          if !is_visible {
            window1.show().expect("Failed to Show window");
          } else {
            window1.hide().expect("Failed to Hide window");
          }
          is_visible = !is_visible;
        }
      }
    }
  });

  app.run();
}
