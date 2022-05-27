use serde::{Deserialize, Serialize};
use skim::prelude::*;
use tauri::{api::shell::open, ShellScope};

use crate::config::{Bookmark, Config, Searcher};

pub struct Launcher {
  config: Config,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchOption {
  rank: i32,
  label: String,
  icon: String,
}

impl SearchOption {
  fn with_rank(other: &SearchOption, rank: i32) -> SearchOption {
    SearchOption {
      rank,
      label: other.label.clone(),
      icon: other.icon.clone(),
    }
  }
}

impl AsRef<str> for SearchOption {
  fn as_ref(&self) -> &str {
    self.label.as_str()
  }
}

impl Into<SearchOption> for &Bookmark {
  fn into(self) -> SearchOption {
    SearchOption {
      rank: 0,
      label: self.label.clone(),
      icon: self.icon.clone(),
    }
  }
}

impl Into<SearchOption> for &Searcher {
  fn into(self) -> SearchOption {
    SearchOption {
      rank: 0,
      label: self.label.clone(),
      icon: self.icon.clone(),
    }
  }
}

impl Launcher {
  pub fn new(config: Config) -> Self {
    Launcher { config }
  }

  fn options_to_receiver(&self) -> SkimItemReceiver {
    let (tx_items, rx_items): (SkimItemSender, SkimItemReceiver) = unbounded();

    let conf = self.config.config.lock().unwrap();
    conf
      .bookmarks
      .iter()
      .map(|(l, bk)| (l, bk.into()))
      .chain(conf.searchers.iter().map(|(l, sh)| (l, sh.into())))
      .for_each(|(_, se): (&String, SearchOption)| {
        let _ = tx_items.send(Arc::new(se));
      });
    drop(tx_items); // indicates that all items have been sent
    rx_items
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
    let receiver = self.options_to_receiver();
    let mut options: Vec<SearchOption> = receiver
      .iter()
      .filter_map(|bk| match fuzzy_engine.match_item(bk.clone()) {
        None => None,
        Some(m) => {
          let rank = m.rank.iter().sum();
          let opt = (*bk).as_any().downcast_ref::<SearchOption>().unwrap();
          Some(SearchOption::with_rank(opt, rank))
        }
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
