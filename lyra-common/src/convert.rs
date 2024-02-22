use anyhow::anyhow;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::engine::Engine as _;
use reqwest::header::CONTENT_TYPE;
use tracing::{error, info};

async fn convert_image(url: String) -> Result<String, anyhow::Error> {
  let resp = reqwest::get(url).await?;
  let ctype = match resp.headers().get(CONTENT_TYPE) {
    Some(v) => v.to_str()?,
    None => return Err(anyhow!("Unknown content type")),
  };
  let ctype = match ctype {
    "image/svg+xml" | "image/png" | "image/vnd.microsoft.icon" | "image/jpeg" => ctype.to_owned(),
    _ => return Err(anyhow!("Unsupported Content Type: {}", ctype)),
  };
  let body = resp.bytes().await?;
  let str = format!("data:{};base64,{}", ctype, BASE64.encode(&body));
  info!("Found: {}", str);
  Ok(str)
}

pub fn decode_bytes(b: &str) -> Result<Vec<u8>, anyhow::Error> {
  BASE64.decode(b).map_err(|e| anyhow!(e))
}

pub async fn image_data_url(url: String) -> Result<String, String> {
  convert_image(url).await.map_err(|err| {
    error!("Failed to parse image to data-url: {}", err);
    "Could not convert image to data-url".into()
  })
}

// TODO: Eventually do away with this as we'll want to just natively
// integrate, but config is currently all data url based. Ideally just use
// PNG since they seem to render cleaner
pub fn parse_image_data(s: &str) -> Option<(String, String)> {
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
