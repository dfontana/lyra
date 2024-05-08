use std::{fmt::Display, str::FromStr};

pub use form_macro::*;

/// TryParse is needed to parse String back to {Self}, which {FromStr} is a blanket impl
/// Validate is needed for types that can be parsed but need another validation (like an int being > 0)
/// Default is _optional_ and will allow FormField<T>::default() to be called without needing to call FormField::new(T::default())
/// Display is needed for FormField::new() since that needs to populate the buffer with an editable String from {Self}
pub trait FormFieldData: Display + TryParse + Validate {}

pub struct FormField<T: FormFieldData> {
  pub buffer: String,
  pub value: Result<T, String>,
}

pub trait TryParse: Sized {
  fn try_parse(v: &String) -> Result<Self, String>;
}

pub trait Validate: Sized {
  fn validate(v: &Self) -> Result<(), String>;
}

impl<T: FormFieldData> FormField<T> {
  pub fn new(value: T) -> FormField<T> {
    let buffer = value.to_string();
    FormField {
      value: Ok(value),
      buffer,
    }
  }

  pub fn parse_and_validate(&mut self) {
    self.value = <T as TryParse>::try_parse(&self.buffer);
    if let Ok(v) = self.value.as_ref() {
      if let Err(err) = <T as Validate>::validate(v) {
        self.value = Err(err);
      }
    }
  }
}

impl<T: Default + Display + FormFieldData> Default for FormField<T> {
  fn default() -> Self {
    FormField::new(T::default())
  }
}

impl<E: Display, F: FromStr<Err = E>> TryParse for F {
  fn try_parse(v: &String) -> Result<Self, String> {
    FromStr::from_str(v).map_err(|err| format!("'{}' could not be parsed; {}", v, err))
  }
}
