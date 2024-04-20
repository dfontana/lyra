use form_macro::FormResult;

struct FieldData<T: Clone> {
  value: Result<T, String>,
}

#[derive(FormResult)]
pub struct LyraSettings {
  field_a: FieldData<String>,
}

fn main() {}
