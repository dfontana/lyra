use anyhow::anyhow;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::engine::Engine as _;
use egui::{Image, Ui};
use reqwest::header::CONTENT_TYPE;
use tracing::info;

pub struct Icon<'a>(Image<'a>);
impl TryFrom<(&str, &str)> for Icon<'_> {
  type Error = anyhow::Error;

  fn try_from((value, label): (&str, &str)) -> Result<Self, Self::Error> {
    parse_image_data(value)
      .ok_or(anyhow!("Cannot render image format {:?}", value))
      .and_then(|(s, ext)| decode_bytes(&s).map(|b| (b, ext)))
      .map(|(bytes, ext)| {
        Icon(Image::from_bytes(
          format!("bytes://{}.{}", label, ext),
          bytes,
        ))
      })
  }
}

impl Icon<'_> {
  pub fn render(self, ui: &mut Ui) {
    ui.add(self.0.maintain_aspect_ratio(true).shrink_to_fit());
  }
}

pub fn data_or_url(value: &str) -> Result<String, anyhow::Error> {
  parse_image_data(value)
    .ok_or(anyhow!("Not a known data url format"))
    .map(|_| value.to_string())
    .or_else(|_| convert_image(value))
}

fn parse_image_data(s: &str) -> Option<(String, String)> {
  let prefixes = vec![
    ("image/svg+xml", "svg"),
    ("image/png", "png"),
    ("image/vnd.microsoft.icon", "ico"),
    ("image/jpeg", "jpg"),
  ];
  prefixes.iter().find_map(|(pf, ext)| {
    s.strip_prefix(&format!("data:{};base64,", pf))
      .map(|s| (s.to_string(), ext.to_string()))
  })
}

fn decode_bytes(b: &str) -> Result<Vec<u8>, anyhow::Error> {
  BASE64.decode(b).map_err(|e| anyhow!(e))
}

fn convert_image(url: &str) -> Result<String, anyhow::Error> {
  let resp = reqwest::blocking::get(url)?;
  let ctype = match resp.headers().get(CONTENT_TYPE) {
    Some(v) => v.to_str()?,
    None => return Err(anyhow!("Unknown content type")),
  };
  let ctype = match ctype {
    "image/svg+xml" | "image/png" | "image/vnd.microsoft.icon" | "image/jpeg" => ctype.to_owned(),
    _ => return Err(anyhow!("Unsupported Content Type: {}", ctype)),
  };
  let body = resp.bytes()?;
  let str = format!("data:{};base64,{}", ctype, BASE64.encode(&body));
  info!("Found: {}", str);
  Ok(str)
}
