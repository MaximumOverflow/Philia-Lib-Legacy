mod search;
mod download;
pub mod danbooru;

use chrono::Utc;
use std::any::Any;
use serde_derive::Deserialize;

pub use search::*;
pub use download::*;
pub type Timestamp = chrono::DateTime<Utc>;

pub trait Post: Any + internal_traits::Search {
	fn md5(&self) -> &str;
	fn score(&self) -> isize;
	fn rating(&self) -> Rating;
	fn resource_url(&self) -> &str;
	fn tags<'l>(&'l self) -> Box<dyn Iterator<Item = &'l str> + 'l>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize)]
pub enum Rating {
	#[serde(alias = "g")]
	General,
	#[serde(alias = "s")]
	Sensitive,
	#[serde(alias = "q")]
	Questionable,
	#[serde(alias = "e")]
	Explicit,
}
