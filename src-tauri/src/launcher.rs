mod commands;
mod searchoption;

use crate::{config::Config, lookup::applookup::AppLookup};
pub use commands::*;
pub use searchoption::{BookmarkOption, SearchOption, SearcherOption};
use skim::prelude::*;

use self::searchoption::Query;

pub struct Launcher {
  config: Config,
  apps: AppLookup,
}

impl Launcher {
  pub fn new(config: Config, apps: AppLookup) -> Self {
    Launcher { config, apps }
  }

  fn options_to_receiver(&self) -> SkimItemReceiver {
    let (tx_items, rx_items): (SkimItemSender, SkimItemReceiver) = unbounded();

    for item in self.apps.iter() {
      println!("{:?}", item);
    }
    let conf = self.config.get();
    conf
      .bookmarks
      .iter()
      .map(|(_, bk)| bk.into())
      .chain(conf.searchers.iter().map(|(_, sh)| sh.into()))
      .for_each(|se: SearchOption| {
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
    options.push(SearchOption::WebQuery(Query::default()));
    options.sort_by(|a, b| b.rank().cmp(&a.rank()));
    options
  }

  pub fn launch(&self, selected: SearchOption) -> Result<(), anyhow::Error> {
    let url = self.config.get_url(&selected)?;
    open::that(url)?;
    Ok(())
  }
}
