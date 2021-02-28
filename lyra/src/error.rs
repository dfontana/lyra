#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// Requested resource not found
  #[error("resource not found: {0}")]
  ResourceNotFound(String),
}

// TODO wry should be expecting a std::error::Error.
//      but here we are :(
impl Into<wry::Error> for Error {
  fn into(self) -> wry::Error {
    match self {
      Self::ResourceNotFound(_) => wry::Error::InitScriptError,
    }
  }
}
