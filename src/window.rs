use crate::error::Error;
use crate::event::{handler, Event};
use include_dir::Dir;
#[cfg(target_os = "macos")]
use wry::application::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
use wry::{
  application::{
    dpi::{LogicalPosition, LogicalSize},
    event_loop::EventLoop,
    system_tray::{SystemTray, SystemTrayBuilder},
    window::{Window, WindowBuilder},
  },
  webview::{WebView, WebViewBuilder},
};

static BUNDLE_DIR: Dir = include_dir!("dist");

pub fn configure() -> Result<(EventLoop<Event>, SystemTray, WebView), wry::Error> {
  let y_offset = 25f64;
  let (disp_w, _) = (1280f64, 800f64);
  let (bar_w, bar_h) = ((disp_w * 0.9f64).floor(), 32f64);
  let (bar_x, bar_y) = (((disp_w - bar_w) / 2f64).floor(), y_offset);

  let mut evloop: EventLoop<Event> = EventLoop::with_user_event();

  // launch macos app without menu and without dock icon
  // shouold be set at launch
  #[cfg(target_os = "macos")]
  evloop.set_activation_policy(ActivationPolicy::Accessory);

  let window = WindowBuilder::new()
    .with_always_on_top(true)
    .with_decorations(false)
    .with_resizable(false)
    .with_visible(false)
    .with_transparent(true)
    .with_position(LogicalPosition::new(bar_x, bar_y))
    .with_inner_size(LogicalSize::new(bar_w, bar_h))
    .build(&evloop)?;

  window.set_visible(false);

  let _webview = WebViewBuilder::new(window)?
    .with_transparent(true)
    .with_rpc_handler(handler)
    .with_custom_protocol("lyra".into(), move |_, path| {
      let mut path = path.to_string().replace("lyra://", "");
      if path.ends_with('/') {
        path.pop();
      }
      let mime = match &path {
        p if p.ends_with(".html") => String::from("text/html"),
        p if p.ends_with(".js") => String::from("text/javascript"),
        p if p.ends_with(".png") => String::from("image/png"),
        p if p.ends_with(".css") => String::from("text/css"),
        p if p.ends_with(".ico") => String::from("img/ico"),
        _ => unimplemented!(),
      };
      BUNDLE_DIR
        .get_file(&path)
        .map(|f| (f.contents().to_vec(), mime))
        .ok_or(Error::ResourceNotFound(path))
        .map_err(|e| {
          eprintln!("Failed to pull resource: {:?}", e);
          e.into()
        })
    })
    .with_url("lyra://index.html")?
    .build()?;

  #[cfg(target_os = "windows")]
  let icon_data = BUNDLE_DIR
    .get_file("static/system_tray.ico")
    .map(|f| f.contents().to_vec())
    .ok_or(Error::ResourceNotFound("static/system_tray.ico".into()));
  #[cfg(target_os = "macos")]
  let icon_data = BUNDLE_DIR
    .get_file("static/system_tray.png")
    .map(|f| f.contents().to_vec())
    .ok_or(Error::ResourceNotFound("static/system_tray.png".into()));
  #[cfg(target_os = "linux")]
  let icon_data = BUNDLE_DIR
    .get_file("static/system_tray.png")
    .map(|f| f.path().to_path_buf())
    .ok_or(Error::ResourceNotFound("static/system_tray.png".into()));

  let icon = icon_data.map_err(|e| {
    eprintln!("Failed to pull resource: {:?}", e);
    wry::Error::InitScriptError
  })?;
  let _system_tray = SystemTrayBuilder::new(icon, None).build(&evloop)?;

  Ok((evloop, _system_tray, _webview))
}
