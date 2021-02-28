use crate::error::Error;
use include_dir::Dir;
use wry::CustomProtocol;

static BUNDLE_DIR: Dir = include_dir!("dist");

pub fn build() -> CustomProtocol {
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
