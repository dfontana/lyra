use windows;

fn main() {
  windows::build!(
    Windows::Win32::WindowsAndMessaging::*,
    Windows::Win32::SystemServices::LRESULT,
    Windows::Win32::Debug::GetLastError,
);
}