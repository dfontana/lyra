use abi_stable::std_types::*;
use glob::glob;
use lyra_plugin::{lyra_plugin, PluginResult};

use anyhow::{anyhow, Context};
use icns::IconFamily;
use plist::Value;
use serde::{Deserialize, Serialize};
use tracing::info;
use std::{
  collections::HashMap,
  fs::File,
  io::BufReader,
  path::{Path, PathBuf},
  sync::Arc,
};

// TODO: Finish this impl
struct AppLookup {
  config: Arc<Config>,
}

#[derive(Debug)]
struct App {
  pub label: String,
  pub icon: String,
  pub path: PathBuf,
}

impl App {
  fn from(p: PathBuf, suffix: &str, icon: String) -> Self {
    App {
      label: p
        .file_name()
        .expect("Glob returned non-file")
        .to_string_lossy()
        .trim_end_matches(suffix)
        .to_string(),
      icon,
      path: p,
    }
  }
}

struct AppLookupIter<T> {
  config: Arc<Config>,
  // Extension to look for in paths
  extension: String,
  // Remaining paths to inspect during iteration
  paths_remaining: Vec<PathBuf>,
  // Current glob being iterated over
  current: Option<glob::Paths>,
  maker: Box<dyn Fn(PathBuf, &str, Arc<Config>) -> T>,
}

impl AppLookup {
  fn new(config: Arc<Config>) -> Self {
    AppLookup { config }
  }

  fn iter(&self) -> AppLookupIter<App> {
    AppLookupIter {
      config: self.config.clone(),
      extension: self.config.app_extension.clone(),
      paths_remaining: self.config.app_paths.clone(),
      current: None,
      maker: Box::new(|p, suffix, cfg| {
        let icon = cfg.get_app_icon(&p).unwrap_or_default();
        App::from(p, suffix, icon)
      }),
    }
  }

  fn init(&self) -> Result<(), anyhow::Error> {
    let items = {
      let apps = AppLookupIter {
        config: self.config.clone(),
        extension: self.config.app_extension.clone(),
        paths_remaining: self.config.app_paths.clone(),
        current: None,
        maker: Box::new(|a, _, _| a),
      };
      apps.collect()
    };
    self.config.update_app_icons(items)
  }
}

impl<T> AppLookupIter<T> {
  fn make_glob(&mut self, p: &Path) -> String {
    format!("{}/*{}", p.display(), self.extension)
  }
}

impl<T> Iterator for AppLookupIter<T> {
  type Item = T;

  fn next(&mut self) -> Option<Self::Item> {
    match self.current.take() {
      None => match self.paths_remaining.pop() {
        None => None,
        Some(next_path) => {
          self.current = Some(glob(self.make_glob(&next_path).as_str()).expect("Failed to glob"));
          self.next()
        }
      },
      Some(mut path) => {
        if let Some(next) = path.next() {
          // There's still more in this path, so restore the value of current
          self.current = Some(path);
          match next {
            // Skip path read errors
            Err(_) => self.next(),
            Ok(item) => Some((self.maker)(item, &self.extension, self.config.clone())),
          }
        } else {
          self.next()
        }
      }
    }
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct Config {
  // TODO: Move these into their own config, separate from the cache
  pub app_paths: Vec<PathBuf>,
  pub app_extension: String,
  #[serde(serialize_with = "toml::ser::tables_last")]
  pub app_icons: HashMap<String, String>,
}

impl Config {
  pub fn get_app_icon(&self, updated: &Path) -> Result<String, anyhow::Error> {
    let key = updated.to_str().unwrap().to_string();
    // TODO Tricky deadlock, but ideally we would persist newly discovered items
    //      rather than waiting until next restart
    Ok(
      self
        .app_icons
        .get(&key)
        .map(|v| v.to_string())
        .get_or_insert_with(|| to_icon(updated).unwrap_or_default())
        .clone(),
    )
  }

  pub fn update_app_icons(&self, updated: Vec<PathBuf>) -> Result<(), anyhow::Error> {
    let new_app_icons = {
      let mut new_app_icons: HashMap<String, String> = updated
        .iter()
        .map(|p| (p.to_str().unwrap().to_string(), p))
        .filter(|(k, _)| !self.app_icons.contains_key(k))
        .map(|(k, p)| (k, to_icon(p).unwrap_or_default()))
        .collect();
      if new_app_icons.is_empty() {
        return Ok(());
      }
      self.app_icons.iter().for_each(|(k, v)| {
        new_app_icons.insert(k.clone(), v.clone());
      });
      new_app_icons
    }; 
    // TODO: Persistance
    // self.app_icons = new_app_icons;
    // self.persist()
    Ok(())
  }
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

fn init(plug_data_dir: &RString, config: &RHashMap<RString, RString>) -> RResult<(), RString>  {
    // TODO: Impl
    // Set up your plugin using the config if necessary. You can store anything needed in the provided
    // plug_data_dir
    // Return RErr if something went wrong
    info!("I inited!");
    
    // Returning this indicates that the plugin initalization is successful
    ROk(())
}

fn query(query: RStr) -> RVec<PluginResult> {
    // TODO: Impl
    let mut result = vec![];
    info!("I queried!");
    
    /* Do stuff here */
    
    RVec::from(result)
}

lyra_plugin!(init, query);
