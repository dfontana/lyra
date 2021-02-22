#![allow(improper_ctypes_definitions)]
use crate::traits::{Event, EventType, Callback, ListenError};
use crate::keycodes::key_from_code;
use std::convert::TryInto;
use cocoa::base::{id,nil};
use cocoa::foundation::NSAutoreleasePool;
use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType, CGEventFlags, EventField};
use std::os::raw::c_void;

pub type CFMachPortRef = *const c_void;
pub type CFIndex = u64;
pub type CFAllocatorRef = id;
pub type CFRunLoopSourceRef = id;
pub type CFRunLoopRef = id;
pub type CFRunLoopMode = id;
pub type CGEventTapProxy = id;
pub type CGEventRef = CGEvent;
pub type CGEventTapPlacement = u32;
pub type CGEventMask = u64;

pub static mut LAST_FLAGS: CGEventFlags = CGEventFlags::CGEventFlagNull;

#[allow(non_upper_case_globals)]
pub const kCGHeadInsertEventTap: u32 = 0;
#[allow(non_upper_case_globals)]
pub const kCGEventMaskForAllEvents: u64 = (1 << CGEventType::LeftMouseDown as u64)
    + (1 << CGEventType::LeftMouseUp as u64)
    + (1 << CGEventType::RightMouseDown as u64)
    + (1 << CGEventType::RightMouseUp as u64)
    + (1 << CGEventType::MouseMoved as u64)
    + (1 << CGEventType::LeftMouseDragged as u64)
    + (1 << CGEventType::RightMouseDragged as u64)
    + (1 << CGEventType::KeyDown as u64)
    + (1 << CGEventType::KeyUp as u64)
    + (1 << CGEventType::FlagsChanged as u64)
    + (1 << CGEventType::ScrollWheel as u64);

#[allow(non_upper_case_globals)]
#[repr(u32)]
pub enum CGEventTapOption {
    ListenOnly = 1,
}

fn default_callback(event: Event) {
    println!("Default {:?}", event)
}
static mut GLOBAL_CALLBACK: Callback = default_callback;

#[cfg(target_os = "macos")]
#[link(name = "Cocoa", kind = "framework")]
extern "C" {
    #[allow(improper_ctypes)]
    pub fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOption,
        eventsOfInterest: CGEventMask,
        callback: QCallback,
        user_info: id,
    ) -> CFMachPortRef;
    pub fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef,
        tap: CFMachPortRef,
        order: CFIndex,
    ) -> CFRunLoopSourceRef;
    pub fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode);
    pub fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    pub fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    pub fn CFRunLoopRun();

    pub static kCFRunLoopCommonModes: CFRunLoopMode;
}

pub type QCallback = unsafe extern "C" fn(
  proxy: CGEventTapProxy,
  _type: CGEventType,
  cg_event: CGEventRef,
  user_info: *mut c_void,
) -> CGEventRef;

unsafe extern "C" fn raw_callback(
    _proxy: CGEventTapProxy,
    _type: CGEventType,
    cg_event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    if let Some(event) = convert(_type, &cg_event) {
        GLOBAL_CALLBACK(event);
    }
    cg_event
}

#[link(name = "Cocoa", kind = "framework")]
pub fn listen(callback: Callback) -> Result<(), ListenError> {
    unsafe {
        GLOBAL_CALLBACK = callback;
        let _pool = NSAutoreleasePool::new(nil);
        let tap = CGEventTapCreate(
            CGEventTapLocation::HID, // HID, Session, AnnotatedSession,
            kCGHeadInsertEventTap,
            CGEventTapOption::ListenOnly,
            kCGEventMaskForAllEvents,
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


pub unsafe fn convert(
  _type: CGEventType,
  cg_event: &CGEvent,
) -> Option<Event> {
  let option_type = match _type {
      CGEventType::KeyDown => {
          let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
          Some(EventType::KeyPress(key_from_code(code.try_into().ok()?)))
      }
      CGEventType::KeyUp => {
          let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
          Some(EventType::KeyRelease(key_from_code(code.try_into().ok()?)))
      }
      CGEventType::FlagsChanged => {
          let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
          let code = code.try_into().ok()?;
          let flags = cg_event.get_flags();
          if flags < LAST_FLAGS {
              LAST_FLAGS = flags;
              Some(EventType::KeyRelease(key_from_code(code)))
          } else {
              LAST_FLAGS = flags;
              Some(EventType::KeyPress(key_from_code(code)))
          }
      }
      _ => None,
  };
  if let Some(event_type) = option_type {
      let name = match event_type {
          EventType::KeyPress(_) => {
              let code =
                  cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u32;
              let flags = cg_event.get_flags();
              Some((code, flags))
          }
          _ => None,
      };
      return Some(Event {
          event_type,
          name,
      });
  }
  None
}