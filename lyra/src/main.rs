extern crate clap;
extern crate daemonize;
#[macro_use]
extern crate include_dir;
extern crate tokio;

mod error;
mod event;
mod hotkeys;
mod window;

use clap::{App, Arg};
use std::fs::File;

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
  let (app, win) = window::configure().expect("Window Setup Failed");
  hotkeys::launch(win);
  app.run();
}
