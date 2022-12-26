use reqwest::header::CONTENT_TYPE;
use anyhow::anyhow;
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
  let str = format!("data:{};base64,{}", ctype, base64::encode(&body));
  info!("Found: {}", str);
  Ok(str)
}

#[tauri::command]
pub async fn image_data_url(url: String) -> Result<String, String> {
  convert_image(url).await.map_err(|err| {
    error!("Failed to parse image to data-url: {}", err);
    "Could not convert image to data-url".into()
  })
}
