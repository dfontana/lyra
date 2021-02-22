mod traits;
mod keycodes;

pub use crate::traits::{
    Callback, ListenError, Event, EventType
};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use crate::macos::{listen as _listen};

pub fn listen(callback: Callback) -> Result<(), ListenError> {
    _listen(callback)
}