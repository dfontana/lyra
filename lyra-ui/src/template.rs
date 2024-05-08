use form::FormFieldData;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Deref, str::FromStr};

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize, FormFieldData)]
#[serde(try_from = "String", into = "String")]
pub struct Template {
  val: String,
  pub markers: usize,
}

impl Template {
  pub fn hydrate(&self, args: &Vec<String>) -> Result<String, TemplateError> {
    if args.len() != self.markers {
      return Err(TemplateError::HydrateError(
        "Not enough args provided to hydrate".into(),
      ));
    }
    let mut hydration = self.val.clone();
    for idx in 0..self.markers {
      let marker = format!("{{{}}}", idx);
      let arg = args
        .get(idx)
        .ok_or_else(|| TemplateError::HydrateError(format!("Missing arg {} for template", idx)))?;
      hydration = hydration.replace(&marker, arg);
    }
    Ok(hydration)
  }
}

impl Deref for Template {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    &self.val
  }
}

impl From<Template> for String {
  fn from(s: Template) -> String {
    s.val
  }
}

impl Display for Template {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.val)
  }
}

#[derive(PartialEq)]
enum State {
  Opened,
  Digit,
  Closed,
}

impl FromStr for Template {
  type Err = TemplateError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut seen = 0x1FF; // Tracking 0-9 as bit vec
    let mut markers = 0;
    let mut state = State::Closed;

    for c in s.chars() {
      match c {
        '{' => {
          if state != State::Closed {
            return Err(TemplateError::InvalidFormat(
              "Missing closing marker: }".into(),
            ));
          }
          state = State::Opened;
        }
        '}' => {
          if state != State::Digit {
            return Err(TemplateError::InvalidFormat(
              "Did not contain number between markers".into(),
            ));
          }
          state = State::Closed;
        }
        k => {
          if state != State::Opened {
            continue;
          }
          let Some(d) = k.to_digit(10) else {
            return Err(TemplateError::InvalidFormat(format!(
              "Marker contains not a digit: {}",
              k
            )));
          };
          if seen & 1 << d == 0 {
            return Err(TemplateError::InvalidFormat(format!(
              "Marker repeated digit: {}",
              d
            )));
          }
          seen ^= 1 << d;
          markers += 1;
          state = State::Digit;
        }
      }
    }

    if 0x1FF != seen + (0..markers).fold(0, |acc, x| acc | 1 << x) {
      return Err(TemplateError::InvalidFormat(
        "Markers are not sequential from 0".into(),
      ));
    }

    if state != State::Closed {
      return Err(TemplateError::InvalidFormat(
        "Missing closing marker: }".into(),
      ));
    }

    Ok(Template {
      val: s.to_string(),
      markers,
    })
  }
}

impl TryFrom<String> for Template {
  type Error = TemplateError;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    value.parse()
  }
}

#[derive(Debug, Eq, PartialEq)]
pub enum TemplateError {
  InvalidFormat(String),
  HydrateError(String),
}
impl Display for TemplateError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TemplateError::InvalidFormat(reason) => f.write_str(reason),
      TemplateError::HydrateError(reason) => f.write_str(reason),
    }
  }
}

impl std::error::Error for TemplateError {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn no_markers() {
    let inp = "https://www.google.com";
    assert_eq!(
      Template::from_str(inp),
      Ok(Template {
        val: inp.to_owned(),
        markers: 0
      })
    );
  }

  #[test]
  fn one_marker() {
    let inp = "https://www.google.com?q={0}";
    assert_eq!(
      Template::from_str(inp),
      Ok(Template {
        val: inp.to_owned(),
        markers: 1
      })
    );
  }

  #[test]
  fn two_marker_reordered() {
    let inp = "https://www.google.com?q={1}&r={0}";
    assert_eq!(
      Template::from_str(inp),
      Ok(Template {
        val: inp.to_owned(),
        markers: 2
      })
    );
  }

  #[test]
  fn missing_closing_brace() {
    let inp = "https://www.google.com?q={1&r={0}";
    assert_eq!(
      Template::from_str(inp),
      Err(TemplateError::InvalidFormat(
        "Missing closing marker: }".into()
      ))
    );
  }

  #[test]
  fn missing_opening_brace() {
    let inp = "https://www.google.com?q=1}&r={0}";
    assert_eq!(
      Template::from_str(inp),
      Err(TemplateError::InvalidFormat(
        "Did not contain number between markers".into()
      ))
    );
  }

  #[test]
  fn missing_closing_brace_end() {
    let inp = "https://www.google.com?q={0";
    assert_eq!(
      Template::from_str(inp),
      Err(TemplateError::InvalidFormat(
        "Missing closing marker: }".into()
      ))
    );
  }

  #[test]
  fn partial_trailing_marker() {
    let inp = "https://www.google.com?q={";
    assert_eq!(
      Template::from_str(inp),
      Err(TemplateError::InvalidFormat(
        "Missing closing marker: }".into()
      ))
    );
  }

  #[test]
  fn nested_braces() {
    let inp = "https://www.google.com?q={{1}}&r={0}";
    assert_eq!(
      Template::from_str(inp),
      Err(TemplateError::InvalidFormat(
        "Missing closing marker: }".into()
      ))
    );
  }

  #[test]
  fn marker_missing_number() {
    let inp = "https://www.google.com?q={0}&r={}";
    assert_eq!(
      Template::from_str(inp),
      Err(TemplateError::InvalidFormat(
        "Did not contain number between markers".into()
      ))
    );
  }

  #[test]
  fn not_contiguous_start() {
    let inp = "https://www.google.com?q={1}";
    assert_eq!(
      Template::from_str(inp),
      Err(TemplateError::InvalidFormat(
        "Markers are not sequential from 0".into()
      ))
    );
  }

  #[test]
  fn not_contiguous_middle() {
    let inp = "https://www.google.com?q={0}&r={2}";
    assert_eq!(
      Template::from_str(inp),
      Err(TemplateError::InvalidFormat(
        "Markers are not sequential from 0".into()
      ))
    );
  }

  #[test]
  fn hydrate_not_enough_args() {
    let inp = vec!["dogs".into()];
    assert_eq!(
      Template::from_str("https://www.google.com?q={0}&r={1}")
        .unwrap()
        .hydrate(&inp),
      Err(TemplateError::HydrateError(
        "Not enough args provided to hydrate".into()
      ))
    );
  }

  #[test]
  fn hydrate_one() {
    let inp = vec!["dogs".into()];
    assert_eq!(
      Template::from_str("https://www.google.com?q={0}")
        .unwrap()
        .hydrate(&inp),
      Ok("https://www.google.com?q=dogs".into())
    );
  }

  #[test]
  fn hydrate_in_order() {
    let inp = vec!["dogs".into(), "cats".into()];
    assert_eq!(
      Template::from_str("https://www.google.com?q={0}&r={1}")
        .unwrap()
        .hydrate(&inp),
      Ok("https://www.google.com?q=dogs&r=cats".into())
    );
  }

  #[test]
  fn hydrate_out_of_order() {
    let inp = vec!["dogs".into(), "cats".into()];
    assert_eq!(
      Template::from_str("https://www.google.com?q={1}&r={0}")
        .unwrap()
        .hydrate(&inp),
      Ok("https://www.google.com?q=cats&r=dogs".into())
    );
  }
}
