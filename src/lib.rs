mod search;
mod download;
pub mod e621;
pub mod rule34;
pub mod danbooru;

use chrono::Utc;
use std::any::Any;
use serde_derive::Deserialize;

pub use search::*;
pub use download::*;
pub type Timestamp = chrono::DateTime<Utc>;

pub trait Post: Any + internal_traits::Search {
	fn id(&self) -> usize;
	fn md5(&self) -> &str;
	fn score(&self) -> isize;
	fn rating(&self) -> Rating;
	fn resource_url(&self) -> &str;
	fn tags<'l>(&'l self) -> Box<dyn Iterator<Item = &'l str> + 'l>;

	fn file_name(&self) -> &str {
		let url = self.resource_url();
		let start = url.rfind(|c| c == '/')
			.map(|i| i + 1)
			.unwrap_or(0);
		&url[start..]
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize)]
pub enum Rating {
	#[serde(alias = "g")]
	#[serde(alias = "general")]
	General,
	#[serde(alias = "s")]
	#[serde(alias = "sensitive")]
	Sensitive,
	#[serde(alias = "q")]
	#[serde(alias = "questionable")]
	Questionable,
	#[serde(alias = "e")]
	#[serde(alias = "explicit")]
	Explicit,
}

const USER_AGENT: &str = const_format::formatcp!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
