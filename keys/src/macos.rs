#![allow(improper_ctypes_definitions)]
use crate::traits::{Action, Callback, Event, Key, Keyset, ListenError};
use cocoa::base::{id, nil};
use cocoa::foundation::NSAutoreleasePool;
use core_graphics::event::{
  CGEvent, CGEventFlags, CGEventTapLocation, CGEventType, CGKeyCode, EventField,
};
use std::{convert::TryInto, ops::BitAnd};
use std::{os::raw::c_void, sync::Mutex};

type CFMachPortRef = *const c_void;
type CFIndex = u64;
type CFAllocatorRef = id;
type CFRunLoopSourceRef = id;
type CFRunLoopRef = id;
type CFRunLoopMode = id;
type CGEventTapProxy = id;
type CGEventRef = CGEvent;
type CGEventTapPlacement = u32;
type CGEventMask = u64;

const ALT: CGKeyCode = 58;
const ALT_GR: CGKeyCode = 61;
const BACKSPACE: CGKeyCode = 51;
const CAPS_LOCK: CGKeyCode = 57;
const CONTROL_LEFT: CGKeyCode = 59;
const DOWN_ARROW: CGKeyCode = 125;
const ESCAPE: CGKeyCode = 53;
const F1: CGKeyCode = 122;
const F10: CGKeyCode = 109;
const F11: CGKeyCode = 103;
const F12: CGKeyCode = 111;
const F2: CGKeyCode = 120;
const F3: CGKeyCode = 99;
const F4: CGKeyCode = 118;
const F5: CGKeyCode = 96;
const F6: CGKeyCode = 97;
const F7: CGKeyCode = 98;
const F8: CGKeyCode = 100;
const F9: CGKeyCode = 101;
const FUNCTION: CGKeyCode = 63;
const LEFT_ARROW: CGKeyCode = 123;
const META_LEFT: CGKeyCode = 55;
const META_RIGHT: CGKeyCode = 54;
const RETURN: CGKeyCode = 36;
const RIGHT_ARROW: CGKeyCode = 124;
const SHIFT_LEFT: CGKeyCode = 56;
const SHIFT_RIGHT: CGKeyCode = 60;
const SPACE: CGKeyCode = 49;
const TAB: CGKeyCode = 48;
const UP_ARROW: CGKeyCode = 126;
const BACK_QUOTE: CGKeyCode = 50;
const NUM1: CGKeyCode = 18;
const NUM2: CGKeyCode = 19;
const NUM3: CGKeyCode = 20;
const NUM4: CGKeyCode = 21;
const NUM5: CGKeyCode = 23;
const NUM6: CGKeyCode = 22;
const NUM7: CGKeyCode = 26;
const NUM8: CGKeyCode = 28;
const NUM9: CGKeyCode = 25;
const NUM0: CGKeyCode = 29;
const MINUS: CGKeyCode = 27;
const EQUAL: CGKeyCode = 24;
const KEY_Q: CGKeyCode = 12;
const KEY_W: CGKeyCode = 13;
const KEY_E: CGKeyCode = 14;
const KEY_R: CGKeyCode = 15;
const KEY_T: CGKeyCode = 17;
const KEY_Y: CGKeyCode = 16;
const KEY_U: CGKeyCode = 32;
const KEY_I: CGKeyCode = 34;
const KEY_O: CGKeyCode = 31;
const KEY_P: CGKeyCode = 35;
const LEFT_BRACKET: CGKeyCode = 33;
const RIGHT_BRACKET: CGKeyCode = 30;
const KEY_A: CGKeyCode = 0;
const KEY_S: CGKeyCode = 1;
const KEY_D: CGKeyCode = 2;
const KEY_F: CGKeyCode = 3;
const KEY_G: CGKeyCode = 5;
const KEY_H: CGKeyCode = 4;
const KEY_J: CGKeyCode = 38;
const KEY_K: CGKeyCode = 40;
const KEY_L: CGKeyCode = 37;
const SEMI_COLON: CGKeyCode = 41;
const QUOTE: CGKeyCode = 39;
const BACK_SLASH: CGKeyCode = 42;
const KEY_Z: CGKeyCode = 6;
const KEY_X: CGKeyCode = 7;
const KEY_C: CGKeyCode = 8;
const KEY_V: CGKeyCode = 9;
const KEY_B: CGKeyCode = 11;
const KEY_N: CGKeyCode = 45;
const KEY_M: CGKeyCode = 46;
const COMMA: CGKeyCode = 43;
const DOT: CGKeyCode = 47;
const SLASH: CGKeyCode = 44;

lazy_static! {
  static ref GLOBAL_CALLBACK: Mutex<Box<Callback>> = Mutex::new(Box::new(|_: Event| {}));
}
const KCG_HEAD_INSERT_EVENT_TAP: u32 = 0;
const KCG_EVENT_MASK_FOR_ALL: u64 = (1 << CGEventType::KeyDown as u64)
  + (1 << CGEventType::KeyUp as u64)
  + (1 << CGEventType::FlagsChanged as u64);

#[repr(u32)]
enum CGEventTapOption {
  ListenOnly = 1,
}

#[cfg(target_os = "macos")]
#[link(name = "Cocoa", kind = "framework")]
extern "C" {
  #[allow(improper_ctypes)]
  fn CGEventTapCreate(
    tap: CGEventTapLocation,
    place: CGEventTapPlacement,
    options: CGEventTapOption,
    eventsOfInterest: CGEventMask,
    callback: QCallback,
    user_info: id,
  ) -> CFMachPortRef;
  fn CFMachPortCreateRunLoopSource(
    allocator: CFAllocatorRef,
    tap: CFMachPortRef,
    order: CFIndex,
  ) -> CFRunLoopSourceRef;
  fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode);
  fn CFRunLoopGetCurrent() -> CFRunLoopRef;
  fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
  fn CFRunLoopRun();

  static kCFRunLoopCommonModes: CFRunLoopMode;
}

type QCallback = unsafe extern "C" fn(
  proxy: CGEventTapProxy,
  _type: CGEventType,
  cg_event: CGEventRef,
  user_info: *mut c_void,
) -> CGEventRef;

#[link(name = "Cocoa", kind = "framework")]
pub fn listen<F>(callback: F) -> Result<(), ListenError>
where
  F: 'static + Fn(Event) + Send,
{
  unsafe {
    {
      let mut cb = GLOBAL_CALLBACK.lock().unwrap();
      *cb = Box::new(callback);
    }
    let _pool = NSAutoreleasePool::new(nil);
    let tap = CGEventTapCreate(
      CGEventTapLocation::HID, // HID, Session, AnnotatedSession,
      KCG_HEAD_INSERT_EVENT_TAP,
      CGEventTapOption::ListenOnly,
      KCG_EVENT_MASK_FOR_ALL,
      raw_callback,
      nil,
    );
    if tap.is_null() {
      return Err(ListenError::EventTapError);
    }
    let _loop = CFMachPortCreateRunLoopSource(nil, tap, 0);
    if _loop.is_null() {
      return Err(ListenError::LoopSourceError);
    }

    let current_loop = CFRunLoopGetCurrent();
    CFRunLoopAddSource(current_loop, _loop, kCFRunLoopCommonModes);

    CGEventTapEnable(tap, true);
    CFRunLoopRun();
  }
  Ok(())
}

unsafe extern "C" fn raw_callback(
  _proxy: CGEventTapProxy,
  _type: CGEventType,
  cg_event: CGEventRef,
  _user_info: *mut c_void,
) -> CGEventRef {
  if let Some(event) = match _type {
    CGEventType::KeyDown => extract_keyset(&cg_event).map(|keyset| Event {
      keyset,
      action: Action::KeyPress,
    }),
    CGEventType::KeyUp => extract_keyset(&cg_event).map(|keyset| Event {
      keyset,
      action: Action::KeyRelease,
    }),
    _ => None,
  } {
    let cb = GLOBAL_CALLBACK.lock().unwrap();
    cb(event);
  }
  cg_event
}

fn extract_keyset(cg_event: &CGEvent) -> Option<Keyset> {
  let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
  let key = key_from_code(code.try_into().ok()?);
  let mods = extract_modifiers(&cg_event.get_flags());
  Some(Keyset { key, mods })
}

fn extract_modifiers(flags: &CGEventFlags) -> Vec<Key> {
  [
    CGEventFlags::CGEventFlagCommand,
    CGEventFlags::CGEventFlagShift,
    CGEventFlags::CGEventFlagAlternate,
    CGEventFlags::CGEventFlagControl,
  ]
  .iter()
  .filter_map(|f| key_from_flag(&flags.bitand(*f)))
  .collect()
}

fn key_from_flag(flag: &CGEventFlags) -> Option<Key> {
  match *flag {
    CGEventFlags::CGEventFlagCommand => Some(Key::MetaLeft),
    CGEventFlags::CGEventFlagShift => Some(Key::ShiftLeft),
    CGEventFlags::CGEventFlagAlternate => Some(Key::Alt),
    CGEventFlags::CGEventFlagControl => Some(Key::ControlLeft),
    _ => None,
  }
}

fn key_from_code(code: CGKeyCode) -> Key {
  match code {
    ALT => Key::Alt,
    ALT_GR => Key::AltGr,
    BACKSPACE => Key::Backspace,
    CAPS_LOCK => Key::CapsLock,
    CONTROL_LEFT => Key::ControlLeft,
    DOWN_ARROW => Key::DownArrow,
    ESCAPE => Key::Escape,
    F1 => Key::F1,
    F10 => Key::F10,
    F11 => Key::F11,
    F12 => Key::F12,
    F2 => Key::F2,
    F3 => Key::F3,
    F4 => Key::F4,
    F5 => Key::F5,
    F6 => Key::F6,
    F7 => Key::F7,
    F8 => Key::F8,
    F9 => Key::F9,
    LEFT_ARROW => Key::LeftArrow,
    META_LEFT => Key::MetaLeft,
    META_RIGHT => Key::MetaRight,
    RETURN => Key::Return,
    RIGHT_ARROW => Key::RightArrow,
    SHIFT_LEFT => Key::ShiftLeft,
    SHIFT_RIGHT => Key::ShiftRight,
    SPACE => Key::Space,
    TAB => Key::Tab,
    UP_ARROW => Key::UpArrow,
    BACK_QUOTE => Key::BackQuote,
    NUM1 => Key::Num1,
    NUM2 => Key::Num2,
    NUM3 => Key::Num3,
    NUM4 => Key::Num4,
    NUM5 => Key::Num5,
    NUM6 => Key::Num6,
    NUM7 => Key::Num7,
    NUM8 => Key::Num8,
    NUM9 => Key::Num9,
    NUM0 => Key::Num0,
    MINUS => Key::Minus,
    EQUAL => Key::Equal,
    KEY_Q => Key::KeyQ,
    KEY_W => Key::KeyW,
    KEY_E => Key::KeyE,
    KEY_R => Key::KeyR,
    KEY_T => Key::KeyT,
    KEY_Y => Key::KeyY,
    KEY_U => Key::KeyU,
    KEY_I => Key::KeyI,
    KEY_O => Key::KeyO,
    KEY_P => Key::KeyP,
    LEFT_BRACKET => Key::LeftBracket,
    RIGHT_BRACKET => Key::RightBracket,
    KEY_A => Key::KeyA,
    KEY_S => Key::KeyS,
    KEY_D => Key::KeyD,
    KEY_F => Key::KeyF,
    KEY_G => Key::KeyG,
    KEY_H => Key::KeyH,
    KEY_J => Key::KeyJ,
    KEY_K => Key::KeyK,
    KEY_L => Key::KeyL,
    SEMI_COLON => Key::SemiColon,
    QUOTE => Key::Quote,
    BACK_SLASH => Key::BackSlash,
    KEY_Z => Key::KeyZ,
    KEY_X => Key::KeyX,
    KEY_C => Key::KeyC,
    KEY_V => Key::KeyV,
    KEY_B => Key::KeyB,
    KEY_N => Key::KeyN,
    KEY_M => Key::KeyM,
    COMMA => Key::Comma,
    DOT => Key::Dot,
    SLASH => Key::Slash,
    FUNCTION => Key::Function,
    code => Key::Unknown(code.into()),
  }
}
