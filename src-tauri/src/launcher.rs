use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use skim::prelude::*;
use tauri::{api::shell::open, ShellScope};

use crate::config::{Bookmark, Config};

pub struct Launcher {
  config: Config,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchOption {
  rank: i32,
  label: String,
  icon: String,
}

impl AsRef<str> for Bookmark {
  fn as_ref(&self) -> &str {
    self.label.as_str()
  }
}

fn options_to_receiver(items: &HashMap<String, Bookmark>) -> SkimItemReceiver {
  let (tx_items, rx_items): (SkimItemSender, SkimItemReceiver) = unbounded();
  items.iter().for_each(|(_, bk)| {
    let _ = tx_items.send(Arc::new(bk.to_owned()));
  });
  drop(tx_items); // indicates that all items have been sent
  rx_items
}

impl Launcher {
  pub fn new(config: Config) -> Self {
    Launcher { config }
  }

  pub async fn get_options(&self, search: &str) -> Vec<SearchOption> {
    if search.is_empty() {
      // Special case, empty string == nothing back instead of everything
      return Vec::new();
    }
    let fuzzy_engine = ExactOrFuzzyEngineFactory::builder()
      .exact_mode(false)
      .fuzzy_algorithm(FuzzyAlgorithm::SkimV2)
      .build()
      .create_engine_with_case(search, CaseMatching::Smart);
    let receiver = options_to_receiver(&(*self.config.config.lock().unwrap()).bookmarks);
    let mut options: Vec<SearchOption> = receiver
      .iter()
      .filter_map(|bk| match fuzzy_engine.match_item(bk.clone()) {
        None => None,
        Some(m) => Some((
          m.rank.iter().sum(),
          (*bk)
            .as_any()
            .downcast_ref::<Bookmark>()
            .unwrap()
            .to_owned(),
        )),
      })
      .map(|(rank, bk)| SearchOption {
        rank,
        label: bk.label.clone(),
        icon: bk.icon.clone(),
      })
      .collect();
    options.sort_by(|a, b| a.rank.cmp(&b.rank));
    options
  }

  pub fn launch(&self, scope: &ShellScope, selected: SearchOption) -> Result<(), anyhow::Error> {
    let url = self.config.get_url_from_label(&selected.label);
    open(scope, url, None)?;
    Ok(())
  }
}
