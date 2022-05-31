use serde::{Deserialize, Serialize};

use crate::config::{Bookmark, Searcher};

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
  pub fn with_rank(other: &SearchOption, rank: i32) -> SearchOption {
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

  pub fn rank(&self) -> i32 {
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
