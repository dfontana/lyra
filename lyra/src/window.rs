use crate::error::Error;
use crate::event;
use include_dir::Dir;
use wry::{Application, Attributes, CustomProtocol, WindowProxy};

static BUNDLE_DIR: Dir = include_dir!("dist");

pub fn configure() -> Result<(Application, WindowProxy), wry::Error> {
  let y_offset = 25f64;
  let (disp_w, _) = (1280f64, 800f64);
  let (bar_w, bar_h) = ((disp_w * 0.9f64).floor(), 32f64);
  let (bar_x, bar_y) = (((disp_w - bar_w) / 2f64).floor(), y_offset);

  let prot = build_protocol();

  let mut app = Application::new()?;
  let attributes = Attributes {
    url: Some("lyra://index.html".to_string()),
    resizable: false,
    visible: false,
    decorations: false,
    transparent: true,
    always_on_top: true,
    width: bar_w,
    height: bar_h,
    x: Some(bar_x),
    y: Some(bar_y),
    skip_taskbar: true,
    ..Default::default()
  };

  let win = app.add_window_with_configs(attributes, Some(Box::new(event::handler)), Some(prot))?;
  Ok((app, win))
}

fn build_protocol() -> CustomProtocol {
  CustomProtocol {
    name: "lyra".into(),
    handler: Box::new(move |path| {
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
    }),
  }
}
