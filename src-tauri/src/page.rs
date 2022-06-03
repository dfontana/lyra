use std::collections::HashMap;

use derive_builder::Builder;
use serde::Serialize;
use serde_json::Value;

#[derive(Default, Builder, Serialize)]
#[builder(setter(into))]
pub struct MainData {
  #[builder(setter(each(name = "call", into)))]
  #[builder(default = "self.default_calls()")]
  calls: HashMap<String, String>,
  #[builder(setter(each(name = "event", into)))]
  #[builder(default = "self.default_events()")]
  events: HashMap<String, String>,
  #[builder(setter(each(name = "style")))]
  #[builder(default)]
  styles: HashMap<String, Value>,
}

impl MainData {
  pub fn builder() -> MainDataBuilder {
    MainDataBuilder::default()
  }
}

impl MainDataBuilder {
  fn default_calls(&self) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("SEARCH".into(), "search".into());
    map.insert("SELECT_SEARCH".into(), "select_searcher".into());
    map.insert("SUBMIT".into(), "submit".into());
    map.insert("CLOSE".into(), "close".into());
    map
  }

  fn default_events(&self) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("RESET".into(), "reset".into());
    map
  }
}

#[derive(Default, Builder, Serialize)]
#[builder(setter(into))]
pub struct SettingsData {
  #[builder(setter(each(name = "call", into)))]
  #[builder(default = "self.default_calls()")]
  calls: HashMap<String, String>,
  #[builder(setter(each(name = "event", into)))]
  #[builder(default)]
  events: HashMap<String, String>,
  #[builder(setter(each(name = "style")))]
  #[builder(default)]
  styles: HashMap<String, Value>,
}

impl SettingsData {
  pub fn builder() -> SettingsDataBuilder {
    SettingsDataBuilder::default()
  }
}

impl SettingsDataBuilder {
  fn default_calls(&self) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("IMAGE_TO_DATA".into(), "image_data_url".into());
    map.insert("SAVE_BOOKMARKS".into(), "save_bookmarks".into());
    map.insert("SAVE_SEARCHERS".into(), "save_searchers".into());
    map.insert("GET_CONFIG".into(), "get_config".into());
    map.insert("VALIDATE_TEMPLATE".into(), "validate_template".into());
    map
  }
}

pub enum Page {
  Settings(SettingsData),
  Main(MainData),
}

impl Page {
  pub fn id(&self) -> &str {
    match self {
      Page::Settings(_) => "lyra-settings",
      Page::Main(_) => "lyra-main",
    }
  }

  pub fn init_script(&self) -> Result<String, anyhow::Error> {
    let data = match self {
      Page::Settings(data) => serde_json::to_value(data)?,
      Page::Main(data) => serde_json::to_value(data)?,
    };
    Ok(format!(
      "window.__LYRA__={};window.__LYRA_PAGE__='{}';window.__LYRA_DEBUG__={}",
      data,
      self.id(),
      cfg!(debug_assertions)
    ))
  }
}
