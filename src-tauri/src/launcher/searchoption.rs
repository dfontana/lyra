use serde::{Deserialize, Serialize};

use crate::{
  config::{Bookmark, Searcher},
  lookup::applookup::App,
};

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
pub struct AppOption {
  pub rank: i32,
  pub label: String,
  pub icon: String,
  pub path: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SearchOption {
  App(AppOption),
  Bookmark(BookmarkOption),
  Searcher(SearcherOption),
  WebQuery(Query),
}

impl SearchOption {
  pub fn with_rank(other: &SearchOption, rank: i32) -> SearchOption {
    match other {
      SearchOption::App(data) => SearchOption::App(AppOption {
        rank,
        label: data.label.clone(),
        path: data.path.clone(),
        icon: data.icon.clone(),
      }),
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
      SearchOption::App(d) => d.rank,
      SearchOption::Bookmark(d) => d.rank,
      SearchOption::Searcher(d) => d.rank,
      SearchOption::WebQuery(d) => d.rank,
    }
  }
}

impl AsRef<str> for SearchOption {
  fn as_ref(&self) -> &str {
    match self {
      SearchOption::App(d) => d.label.as_str(),
      SearchOption::Bookmark(d) => d.shortname.as_str(),
      SearchOption::Searcher(d) => d.shortname.as_str(),
      SearchOption::WebQuery(d) => d.label.as_str(),
    }
  }
}

impl From<&Bookmark> for SearchOption {
  fn from(bk: &Bookmark) -> SearchOption {
    SearchOption::Bookmark(BookmarkOption {
      rank: 0,
      label: bk.label.clone(),
      shortname: bk.shortname.clone(),
      icon: bk.icon.clone(),
    })
  }
}

impl From<&Searcher> for SearchOption {
  fn from(sh: &Searcher) -> SearchOption {
    SearchOption::Searcher(SearcherOption {
      rank: 0,
      label: sh.label.clone(),
      shortname: sh.shortname.clone(),
      icon: sh.icon.clone(),
      required_args: sh.template.markers,
      args: Vec::new(),
    })
  }
}

impl From<App> for SearchOption {
  fn from(app: App) -> SearchOption {
    SearchOption::App(AppOption {
      rank: 0,
      label: app.label.clone(),
      path: app.path.to_string_lossy().to_string(),
      icon: app.icon.clone(),
    })
  }
}

impl Default for Query {
  fn default() -> Self {
    Self {
      rank: i32::MAX,
      label: "Search the Web".into(),
      query: "".into(),
    }
  }
}
