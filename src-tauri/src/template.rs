use std::{fmt::Display, ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::launcher::SearcherOption;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Template(String);

impl Template {
  pub fn hydrate(&self, opt: &SearcherOption) -> Result<String, anyhow::Error> {
    todo!("Impl, be wary of opt.args != opt.requiredArgs != template spaces")
  }
}

impl Deref for Template {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Template> for String {
  fn from(s: Template) -> String {
    s.0
  }
}

impl FromStr for Template {
  type Err = TemplateError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.len() != 13 {
      todo!("Verify actual logic, not 13. Eg at least one template, it should be numbered, and all nums unique")
      // Err(TemplateError::InvalidFormat("Length".into()))
    } else {
      Ok(Template(s.to_string()))
    }
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
}
impl Display for TemplateError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TemplateError::InvalidFormat(reason) => f.write_str(reason),
    }
  }
}

impl std::error::Error for TemplateError {}
