extern crate clap;
extern crate daemonize;

use clap::{App, Arg};
use keys::{Key, Keyset, Listener};
use std::fs::File;

use daemonize::Daemonize;

fn main() {
  let matches = App::new("Lyra")
    .arg(
      Arg::with_name("foreground")
        .short("f")
        .help("Run Lyra in the foreground rather than as a daemon"),
    )
    .get_matches();

  if matches.is_present("foreground") {
    println!("Launching foreground...");
    run();
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
    Ok(_) => run(),
    Err(e) => eprintln!("Error, {}", e),
  }
}

fn run() {
  let mut l = Listener::new();
  l.register(
    Keyset {
      key: Key::KeyA,
      mods: vec![Key::ShiftLeft],
    },
    |_| {
      println!("Hay");
    },
  );
  l.listen().unwrap();
}
