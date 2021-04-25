#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// Requested resource not found
  #[error("resource not found: {0}")]
  ResourceNotFound(String),

  /// RPC Event Creation Failed
  #[error("Event could not be interpreted: {0}")]
  RpcEventFailure(String),
}

// TODO wry should be expecting a std::error::Error.
//      but here we are :(
impl Into<wry::Error> for Error {
  fn into(self) -> wry::Error {
    match self {
      Self::ResourceNotFound(_) => wry::Error::InitScriptError,
      Self::RpcEventFailure(a) => wry::Error::RpcScriptError("Call".to_string(), a),
    }
  }
}
