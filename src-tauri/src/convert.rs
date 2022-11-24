use std::{fs::File, io::BufReader, path::Path};

use anyhow::{anyhow, Context};
use icns::IconFamily;
use plist::Value;
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
  let str = format!("data:{};base64,{}", ctype, base64::encode(&body));
  info!("Found: {}", str);
  Ok(str)
}

pub fn to_icon(p: &Path) -> Result<String, anyhow::Error> {
  let icns = Value::from_file(p.join("Contents/info.plist"))
    .map_err(|e| anyhow!("Failed to get plist for {:?}: {}", p, e))?
    .as_dictionary()
    .and_then(|dict| dict.get("CFBundleIconFile"))
    .map(|v| {
      let name = v.as_string().unwrap_or_default();
      let norm = if !name.ends_with(".icns") {
        name.to_owned() + ".icns"
      } else {
        name.to_string()
      };
      p.join("Contents/Resources").join(norm)
    })
    .ok_or_else(|| anyhow!("No CFBundleIconFile in plist: {:?}", p))?;

  let icon_family = IconFamily::read(BufReader::new(
    File::open(&icns).context(format!("Failed to open: {:?}", icns))?,
  ))?;
  let icon_type = if icon_family.has_icon_with_type(icns::IconType::RGBA32_64x64) {
    icns::IconType::RGBA32_64x64
  } else {
    *icon_family
      .available_icons()
      .iter()
      .last()
      .ok_or_else(|| anyhow!("No icns for file {:?}", p))?
  };

  let mut out: Vec<u8> = Vec::new();
  let image = icon_family.get_icon_with_type(icon_type)?;
  image.write_png(&mut out)?;
  Ok(format!("data:image/png;base64,{}", base64::encode(&out)))
}

#[tauri::command]
pub async fn image_data_url(url: String) -> Result<String, String> {
  convert_image(url).await.map_err(|err| {
    error!("Failed to parse image to data-url: {}", err);
    "Could not convert image to data-url".into()
  })
}
