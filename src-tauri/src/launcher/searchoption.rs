use serde::{Deserialize, Serialize};

use crate::config::{Bookmark, Searcher};

#[derive(Debug, Deserialize, Serialize)]
pub struct BookmarkOption {
  pub rank: i32,
  pub label: String,
  pub shortname: String,
  pub icon: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearcherOption {
  pub rank: i32,
  pub label: String,
  pub shortname: String,
  pub icon: String,
  pub required_args: usize,
  pub args: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Query {
  pub rank: i32,
  pub label: String,
  pub query: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SearchOption {
  Bookmark(BookmarkOption),
  Searcher(SearcherOption),
  WebQuery(Query),
}

impl SearchOption {
  pub fn with_rank(other: &SearchOption, rank: i32) -> SearchOption {
    match other {
      SearchOption::Bookmark(data) => SearchOption::Bookmark(BookmarkOption {
        rank,
        label: data.label.clone(),
        shortname: data.shortname.clone(),
        icon: data.icon.clone(),
      }),
      SearchOption::Searcher(data) => SearchOption::Searcher(SearcherOption {
        rank,
        label: data.label.clone(),
        shortname: data.shortname.clone(),
        icon: data.icon.clone(),
        required_args: data.required_args,
        args: data.args.clone(),
      }),
      SearchOption::WebQuery(query) => SearchOption::WebQuery(query.clone()),
    }
  }

  pub fn rank(&self) -> i32 {
    match self {
      SearchOption::Searcher(d) => d.rank,
      SearchOption::Bookmark(d) => d.rank,
      SearchOption::WebQuery(d) => d.rank,
    }
  }
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

impl Into<SearchOption> for &Bookmark {
  fn into(self) -> SearchOption {
    SearchOption::Bookmark(BookmarkOption {
      rank: 0,
      label: self.label.clone(),
      shortname: self.shortname.clone(),
      icon: self.icon.clone(),
    })
  }
}

impl Into<SearchOption> for &Searcher {
  fn into(self) -> SearchOption {
    SearchOption::Searcher(SearcherOption {
      rank: 0,
      label: self.label.clone(),
      shortname: self.shortname.clone(),
      icon: self.icon.clone(),
      required_args: self.template.markers,
      args: Vec::new(),
    })
  }
}

impl Default for Query {
  fn default() -> Self {
    Self {
      rank: i32::MIN,
      label: "Search the Web".into(),
      query: "".into(),
    }
  }
}
