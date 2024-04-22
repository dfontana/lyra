use std::{fmt::Display, str::FromStr};

pub use form_macro::*;

pub trait FormFieldData: Clone + Display + TryParse + Validate {}

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

impl<T: Display + FormFieldData> FormField<T> {
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

impl<E: Display, F: FromStr<Err = E> + Display> TryParse for F {
  fn try_parse(v: &String) -> Result<Self, String> {
    FromStr::from_str(v).map_err(|err| format!("'{}' could not be parsed; {}", v, err))
  }
}
