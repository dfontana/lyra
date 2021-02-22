#[macro_use]
extern crate lazy_static;
mod traits;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

pub use crate::traits::{Action, Callback, Event, Key, Keyset, ListenError};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use crate::macos::listen as _listen;

type CallbackMap = Arc<Mutex<HashMap<Keyset, Box<Callback>>>>;

pub struct Listener {
  callbacks: CallbackMap,
}

impl Listener {
  pub fn new() -> Self {
    Listener {
      callbacks: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub fn register<F>(&mut self, keys: Keyset, callback: F) -> &mut Self
  where
    F: 'static + Fn(Event) + Send,
  {
    self
      .callbacks
      .lock()
      .unwrap()
      .insert(keys, Box::new(callback));
    self
  }

  pub fn listen(self) -> Result<(), ListenError> {
    _listen(move |e| {
      if e.action == Action::KeyPress {
        // TODO let this be a config on the register
        return;
      }
      if let Some(cb) = self.callbacks.lock().unwrap().get(&e.keyset) {
        cb(e)
      }
    })
  }
}
