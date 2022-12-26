mod commands;
mod searchoption;

use std::collections::HashMap;

use crate::config::{Config, LoadedPlugin};
use abi_stable::std_types::{RStr, Tuple2};
pub use commands::*;
pub use searchoption::{BookmarkOption, SearchOption, SearcherOption};
use serde::{Deserialize, Serialize};
use skim::prelude::*;

use self::searchoption::Query;

pub struct Launcher {
  config: Arc<Config>,
  plugins: HashMap<String, LoadedPlugin>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SortWrapper {
  rank: i32,
  matchable: String,
  data: HashMap<String, String>,
  source: String,
}

impl SkimItem for SortWrapper {
  fn text(&self) -> Cow<str> {
    Cow::from(self.matchable.clone())
  }
}

impl Launcher {
  pub fn new(config: Arc<Config>, plugins: HashMap<String, LoadedPlugin>) -> Self {
    Launcher { config, plugins }
  }

  fn options_to_receiver(&self, query: &str) -> SkimItemReceiver {
    let (tx_items, rx_items): (SkimItemSender, SkimItemReceiver) = unbounded();
    let conf = self.config.get();

    for item in conf.bookmarks.values() {
      let _ = tx_items.send(Arc::new(SortWrapper {
        rank: 0,
        matchable: item.shortname.clone(),
        data: item.to_map(),
        source: "SearchOptionBookmark".to_string(),
      }));
    }

    for item in conf.searchers.values() {
      let _ = tx_items.send(Arc::new(SortWrapper {
        rank: 0,
        matchable: item.shortname.clone(),
        data: item.to_map(),
        source: "SearchOptionSearcher".to_string(),
      }));
    }

   for lp in self.plugins.values() {
       if lp.prefix.is_empty() {
           continue;
       }

       for pr in unsafe { lp.plugin.invoke_query(RStr::from_str(query)) }.iter() {
            let mut data: HashMap<String, String> = HashMap::new();
            for Tuple2(k, v) in pr.data.iter() {
              data.insert(k.to_string(), v.to_string());
            }
            let sw = SortWrapper {
              rank: 0,
              matchable: pr.matchable.to_string(),
              data,
              source: lp.source.clone(),
            };
            let _ = tx_items.send(Arc::new(sw));
       }
   }

    drop(tx_items); // indicates that all items have been sent
    rx_items
  }

  pub async fn get_options(&self, search: &str) -> Vec<SortWrapper> {
    if search.is_empty() {
      // Special case, empty string == nothing back instead of everything
      return Vec::new();
    }
    let fuzzy_engine = ExactOrFuzzyEngineFactory::builder()
      .exact_mode(false)
      .fuzzy_algorithm(FuzzyAlgorithm::SkimV2)
      .build()
      .create_engine_with_case(search, CaseMatching::Smart);
    let receiver = self.options_to_receiver(search);
    let mut options: Vec<SortWrapper> = receiver
      .iter()
      .filter_map(|bk| match fuzzy_engine.match_item(bk.clone()) {
        None => None,
        Some(m) => {
          let mut opt = (*bk)
            .as_any()
            .downcast_ref::<SortWrapper>()
            .unwrap()
            .clone();
          opt.rank = m.rank.iter().sum();
          Some(opt)
        }
      })
      .collect();
    options.sort_by_cached_key(|sw| sw.rank);
    options.truncate(self.config.get().result_count);
    options.push(SortWrapper {
      rank: 0,
      matchable: "".into(),
      data: Query::default().to_map(),
      source: "SearchOptionWebQuery".into(),
    });
    options
  }

  pub fn launch(&self, selected: SortWrapper) -> Result<(), anyhow::Error> {
    // TODO: delegate to plugin
    // let url = self.config.get_url(&selected)?;
    // open::that(url)?;
    Ok(())
  }
}
