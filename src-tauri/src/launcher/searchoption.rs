use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config::{Bookmark, Searcher};

#[derive(Debug, Deserialize, Serialize)]
pub struct BookmarkOption {
  pub label: String,
  pub shortname: String,
  pub icon: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearcherOption {
  pub label: String,
  pub shortname: String,
  pub icon: String,
  pub required_args: usize,
  pub args: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Query {
  pub label: String,
  pub query: String,
}

// #[derive(Debug, Deserialize, Serialize)]
// pub struct AppOption {
//   pub label: String,
//   pub icon: String,
//   pub path: String,
// }

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SearchOption {
  Bookmark(BookmarkOption),
  Searcher(SearcherOption),
  WebQuery(Query),
}

impl AsRef<str> for SearchOption {
  fn as_ref(&self) -> &str {
    match self {
      SearchOption::Bookmark(d) => d.shortname.as_str(),
      SearchOption::Searcher(d) => d.shortname.as_str(),
      SearchOption::WebQuery(d) => d.label.as_str(),
    }
  }
}

impl From<&Bookmark> for SearchOption {
  fn from(bk: &Bookmark) -> SearchOption {
    SearchOption::Bookmark(BookmarkOption {
      label: bk.label.clone(),
      shortname: bk.shortname.clone(),
      icon: bk.icon.clone(),
    })
  }
}

impl Bookmark {
    pub fn to_map(&self) -> HashMap<String, String> {
        // TODO: Impl
        todo!()
    }
}

impl From<&Searcher> for SearchOption {
  fn from(sh: &Searcher) -> SearchOption {
    SearchOption::Searcher(SearcherOption {
      label: sh.label.clone(),
      shortname: sh.shortname.clone(),
      icon: sh.icon.clone(),
      required_args: sh.template.markers,
      args: Vec::new(),
    })
  }
}

impl Searcher {
    pub fn to_map(&self) -> HashMap<String, String> {
        // TODO: Impl
        todo!()
    }
}

impl Query {
    pub fn to_map(&self) -> HashMap<String, String> {
        // TODO: Impl
        todo!()
    }
}

impl Default for Query {
  fn default() -> Self {
    Self {
      label: "Search the Web".into(),
      query: "".into(),
    }
  }
}
