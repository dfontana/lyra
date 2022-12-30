use calc::Context;
use serde::Serialize;

#[derive(Serialize)]
pub struct CalcError {
  message: String,
  start: usize,
  end: usize,
}

#[tauri::command]
pub fn calculate(expression: String) -> Result<String, CalcError> {
  let mut context = Context::<f64>::default();
  context
    .evaluate_annotated(&expression)
    .map_err(|err| match err {
      calc::Error::Parse(err) => match err {
        lalrpop_util::ParseError::InvalidToken { location } => CalcError {
          message: "Invalid Token".into(),
          start: location,
          end: location,
        },
        lalrpop_util::ParseError::UnrecognizedEOF { location, .. } => CalcError {
          message: "Unfinished Expression".into(),
          start: location,
          end: location,
        },
        lalrpop_util::ParseError::UnrecognizedToken {
          token: (start, _token, end),
          ..
        } => CalcError {
          message: "Unknown Token".into(),
          start,
          end,
        },
        lalrpop_util::ParseError::ExtraToken {
          token: (start, _token, end),
          ..
        } => CalcError {
          message: "Extra Token".into(),
          start,
          end,
        },
        lalrpop_util::ParseError::User { error } => CalcError {
          message: format!("Unknown Error {}", error),
          start: 0,
          end: 0,
        },
      },
      _ => CalcError {
        message: format!("Unknown Error {}", err),
        start: 0,
        end: 0,
      },
    })
}
