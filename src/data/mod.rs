use chrono::Utc;
use std::any::Any;
use std::iter::Map;
use std::slice::Iter;
use serde_derive::Deserialize;
use crate::data::internal::EnablePost;

pub mod e621;
pub mod rule34;
pub mod danbooru;

pub type Timestamp = chrono::DateTime<Utc>;

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

pub trait Post: Any + EnablePost + Into<GenericPost> {
	type TagIterator<'l>: Iterator<Item = &'l str>;

	fn id(&self) -> usize;
	fn md5(&self) -> &str;
	fn score(&self) -> isize;
	fn rating(&self) -> Rating;
	fn resource_url(&self) -> &str;
	fn tags(&self) -> Self::TagIterator<'_>;

	fn file_name(&self) -> &str {
		let url = self.resource_url();
		let start = url.rfind(|c| c == '/').map(|i| i + 1).unwrap_or(0);
		&url[start..]
	}
	
	fn file_ext(&self) -> &str {
		let url = self.resource_url();
		let start = url.rfind(|c| c == '.').map(|i| i + 1).unwrap_or(0);
		&url[start..]
	}
	
	fn tags_owned(&self) -> Vec<String> {
		self.tags().map(|t| t.to_string()).collect()
	}
}

#[derive(Debug, Clone)]
pub struct GenericPost {
	pub id: usize,
	pub md5: String,
	pub score: isize,
	pub rating: Rating,
	pub tags: Vec<String>,
	pub resource_url: String,
}

impl Post for GenericPost {
	type TagIterator<'l> = Map<Iter<'l, String>, fn(&String)->&str>;

	fn id(&self) -> usize {
		self.id
	}

	fn md5(&self) -> &str {
		&self.md5
	}

	fn score(&self) -> isize {
		self.score
	}

	fn rating(&self) -> Rating {
		self.rating
	}

	fn resource_url(&self) -> &str {
		&self.resource_url
	}

	fn tags(&self) -> Self::TagIterator<'_> {
		self.tags.iter().map(|t| t.as_str())
	}
}

mod internal {
	pub trait EnablePost {}
	impl EnablePost for crate::e621::Post {}
	impl EnablePost for crate::rule34::Post {}
	impl EnablePost for crate::danbooru::Post {}
	impl EnablePost for crate::data::GenericPost {}
}
