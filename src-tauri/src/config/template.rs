use crate::launcher::SearcherOption;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Deref, str::FromStr};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Template {
  val: String,
  markers: usize,
}

impl Template {
  pub fn hydrate(&self, opt: &SearcherOption) -> Result<String, TemplateError> {
    if opt.args.len() != self.markers {
      return Err(TemplateError::HydrateError(
        "Not enough args provided to hydrate".into(),
      ));
    }
    let mut hydration = self.val.clone();
    for idx in 0..self.markers {
      let marker = format!("{{{}}}", idx);
      let arg = opt
        .args
        .get(idx)
        .ok_or(TemplateError::HydrateError(format!(
          "Missing arg {} for template",
          idx
        )))?;
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

impl FromStr for Template {
  type Err = TemplateError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut seen = 0x1FF;
    let mut markers = 0;
    let mut is_next = false;
    let mut has_open = false;
    for c in s.chars() {
      match c {
        '{' => {
          if has_open || is_next {
            return Err(TemplateError::InvalidFormat(
              "Missing closing marker: }".into(),
            ));
          } else {
            has_open = true;
            is_next = true;
          }
        }
        '}' => {
          if !has_open {
            return Err(TemplateError::InvalidFormat(
              "Missing opening marker: {".into(),
            ));
          } else if is_next {
            return Err(TemplateError::InvalidFormat(
              "Did not contain number between markers".into(),
            ));
          } else {
            has_open = false;
          }
        }
        k => {
          if !is_next {
            continue;
          }
          if let Some(d) = k.to_digit(10) {
            if seen & 1 << d == 0 {
              return Err(TemplateError::InvalidFormat(format!(
                "Marker repeated digit: {}",
                d
              )));
            } else {
              seen ^= 1 << d;
              markers += 1;
            }
            is_next = false;
          } else {
            return Err(TemplateError::InvalidFormat(format!(
              "Marker contains not a digit: {}",
              k
            )));
          }
        }
      }
    }

    if seen == 0x1FF {
      return Err(TemplateError::InvalidFormat(
        "Must contain at least one marker in template".into(),
      ));
    }
    if 0x1FF != seen + (0..markers).fold(0, |acc, x| acc | 1 << x) {
      return Err(TemplateError::InvalidFormat(
        "Markers are not sequential from 0".into(),
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

#[derive(Debug, PartialEq)]
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
      Err(TemplateError::InvalidFormat(
        "Must contain at least one marker in template".into()
      ))
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
        "Missing opening marker: {".into()
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
    let inp = SearcherOption {
      icon: "".into(),
      rank: 0,
      label: "".into(),
      required_args: 2,
      args: vec!["dogs".into()],
    };
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
    let inp = SearcherOption {
      icon: "".into(),
      rank: 0,
      label: "".into(),
      required_args: 1,
      args: vec!["dogs".into()],
    };
    assert_eq!(
      Template::from_str("https://www.google.com?q={0}")
        .unwrap()
        .hydrate(&inp),
      Ok("https://www.google.com?q=dogs".into())
    );
  }

  #[test]
  fn hydrate_in_order() {
    let inp = SearcherOption {
      icon: "".into(),
      rank: 0,
      label: "".into(),
      required_args: 2,
      args: vec!["dogs".into(), "cats".into()],
    };
    assert_eq!(
      Template::from_str("https://www.google.com?q={0}&r={1}")
        .unwrap()
        .hydrate(&inp),
      Ok("https://www.google.com?q=dogs&r=cats".into())
    );
  }

  #[test]
  fn hydrate_out_of_order() {
    let inp = SearcherOption {
      icon: "".into(),
      rank: 0,
      label: "".into(),
      required_args: 2,
      args: vec!["dogs".into(), "cats".into()],
    };
    assert_eq!(
      Template::from_str("https://www.google.com?q={1}&r={0}")
        .unwrap()
        .hydrate(&inp),
      Ok("https://www.google.com?q=cats&r=dogs".into())
    );
  }
}
