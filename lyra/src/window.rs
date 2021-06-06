use crate::error::Error;
use crate::event::{handler, Event};
use include_dir::Dir;
use wry::{
  application::{
    dpi::{LogicalPosition, LogicalSize},
    event_loop::EventLoop,
    menu::MenuItem,
    platform::system_tray::SystemTrayBuilder,
    window::WindowBuilder,
  },
  webview::{WebView, WebViewBuilder},
};

static BUNDLE_DIR: Dir = include_dir!("dist");

pub fn configure() -> Result<(EventLoop<Event>, WebView), wry::Error> {
  let y_offset = 25f64;
  let (disp_w, _) = (1280f64, 800f64);
  let (bar_w, bar_h) = ((disp_w * 0.9f64).floor(), 32f64);
  let (bar_x, bar_y) = (((disp_w - bar_w) / 2f64).floor(), y_offset);

  let evloop: EventLoop<Event> = EventLoop::with_user_event();
  let window = WindowBuilder::new()
    .with_always_on_top(true)
    .with_decorations(false)
    .with_resizable(false)
    .with_visible(false)
    .with_transparent(true)
    .with_position(LogicalPosition::new(bar_x, bar_y))
    .with_inner_size(LogicalSize::new(bar_w, bar_h))
    .build(&evloop)?;

  let _webview = WebViewBuilder::new(window)?
    .with_transparent(true)
    .with_rpc_handler(handler)
    .with_custom_protocol("lyra".into(), move |_, path| {
      let mut path = path.to_string().replace("lyra://", "");
      if path.ends_with('/') {
        path.pop();
      }
      BUNDLE_DIR
        .get_file(&path)
        .map(|f| f.contents().to_vec())
        .ok_or(Error::ResourceNotFound(path))
        .map_err(|e| {
          eprintln!("Failed to pull resource: {:?}", e);
          e.into()
        })
    })
    .with_url("lyra://index.html")?
    .build()?;

  // TODO Linux requires a pathbuf apparently, but that doesn't jive with our getup. Will need a fix
  #[cfg(target_os = "windows")]
  let icon_path = "static/system_tray.ico";
  #[cfg(target_os = "macos")]
  let icon_path = "static/system_tray.png";

  let icon = BUNDLE_DIR
    .get_file(icon_path)
    .map(|f| f.contents().to_vec())
    .ok_or(Error::ResourceNotFound(icon_path.into()))
    .map_err(|e| {
      eprintln!("Failed to pull resource: {:?}", e);
      wry::Error::InitScriptError 
    })?;
  let open_new_window = MenuItem::new("Lyra");
  let _system_tray = SystemTrayBuilder::new(icon, vec![open_new_window]).build(&evloop)?;

  Ok((evloop, _webview))
}
