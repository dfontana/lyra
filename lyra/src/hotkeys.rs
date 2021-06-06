use crate::event::Event;
use keys::{Key, Keyset, Listener};
use tokio::{sync::broadcast, task};
use wry::application::event_loop::EventLoopProxy;

pub fn launch(proxy: EventLoopProxy<Event>) {
  let (tx, mut rx) = broadcast::channel(16);
  task::spawn(async move {
    println!("[send] Launching listener");
    Listener::new()
      // TODO: Register ESC (for closing the window)
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
          // TODO match on v (keyset) to determine action to take. Should probably store these globally.
          if !is_visible {
            proxy
              .send_event(Event::Show)
              .expect("Failed to Show window");
          } else {
            proxy
              .send_event(Event::Hide)
              .expect("Failed to Hide window");
          }
          is_visible = !is_visible;
        }
      }
    }
  });
}
