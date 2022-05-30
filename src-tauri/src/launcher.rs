use serde::{Deserialize, Serialize};
use skim::prelude::*;

use crate::config::{Bookmark, Config, Searcher};

pub struct Launcher {
  config: Config,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BookmarkOption {
  pub rank: i32,
  pub label: String,
  pub icon: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearcherOption {
  pub rank: i32,
  pub label: String,
  pub icon: String,
  pub required_args: usize,
  pub args: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SearchOption {
  Bookmark(BookmarkOption),
  Searcher(SearcherOption),
}

impl SearchOption {
  fn with_rank(other: &SearchOption, rank: i32) -> SearchOption {
    match other {
      SearchOption::Bookmark(data) => SearchOption::Bookmark(BookmarkOption {
        rank,
        label: data.label.clone(),
        icon: data.icon.clone(),
      }),
      SearchOption::Searcher(data) => SearchOption::Searcher(SearcherOption {
        rank,
        label: data.label.clone(),
        icon: data.icon.clone(),
        required_args: data.required_args,
        args: data.args.clone(),
      }),
    }
  }

  fn rank(&self) -> i32 {
    match self {
      SearchOption::Searcher(d) => d.rank,
      SearchOption::Bookmark(d) => d.rank,
    }
  }
}

impl AsRef<str> for SearchOption {
  fn as_ref(&self) -> &str {
    match self {
      SearchOption::Bookmark(d) => d.label.as_str(),
      SearchOption::Searcher(d) => d.label.as_str(),
    }
  }
}

impl Into<SearchOption> for &Bookmark {
  fn into(self) -> SearchOption {
    SearchOption::Bookmark(BookmarkOption {
      rank: 0,
      label: self.label.clone(),
      icon: self.icon.clone(),
    })
  }
}

impl Into<SearchOption> for &Searcher {
  fn into(self) -> SearchOption {
    SearchOption::Searcher(SearcherOption {
      rank: 0,
      label: self.label.clone(),
      icon: self.icon.clone(),
      required_args: self.arg_count,
      args: Vec::new(),
    })
  }
}

impl Launcher {
  pub fn new(config: Config) -> Self {
    Launcher { config }
  }

  fn options_to_receiver(&self) -> SkimItemReceiver {
    let (tx_items, rx_items): (SkimItemSender, SkimItemReceiver) = unbounded();

    let conf = self.config.get();
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
    options.sort_by(|a, b| a.rank().cmp(&b.rank()));
    options
  }

  pub fn launch(&self, selected: SearchOption) -> Result<(), anyhow::Error> {
    let url = self.config.get_url(&selected)?;
    open::that(url)?;
    Ok(())
  }
}
