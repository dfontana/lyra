extern crate daemonize;

use hotkey::{Listener, HotkeyListener, ListenerHotkey, modifiers};
use keys::{listen, Event, EventType};
use std::{env::args, fs::File};

use daemonize::Daemonize;

fn main() {
    run()
    // let stdout = File::create("/tmp/daemon.out").unwrap();
    // let stderr = File::create("/tmp/daemon.err").unwrap();

    // let daemonize = Daemonize::new()
    //     .pid_file("/tmp/daemon.pid")
    //     .stdout(stdout)
    //     .stderr(stderr)
    //     .exit_action(|| println!("Executed before master process exits"));

    // match daemonize.start() {
    //     Ok(_) => run(),
    //     Err(e) => eprintln!("Error, {}", e),
    // }
}

fn run() {
    println!("Starting Daemon...");
    // let hotkey1 = ListenerHotkey::new(modifiers::ALT, keys::D);
    // let mut hk = Listener::new();
    // hk.register_hotkey(
    //     hotkey1,
    //     || println!("Super-Alt-Space pressed!"),
    // )
    // .unwrap();
    // loop {}

    loop {
        if let Err(error) = listen(callback) {
            println!("Error: {:?}", error)
        }
    }
}

fn callback(event: Event) {
    match event.event_type {
        EventType::KeyPress(key) => println!("User wrote {:?}", key),
        _ => ()
    }
}