use std::str::Split;
use serde_derive::Deserialize;
use crate::data::Rating;

#[derive(Debug, Deserialize)]
pub struct Post {
	pub id: usize,
	pub directory: usize,
	pub hash: String,
	pub width: usize,
	pub height: usize,
	pub image: String,
	pub change: usize,
	pub owner: String,
	pub parent_id: usize,
	pub rating: Rating,
	pub sample: usize,
	pub sample_width: usize,
	pub sample_height: usize,
	pub score: isize,
	pub tags: String,
	pub file_url: String,
	pub sample_url: String,
	pub preview_url: String,
}

impl crate::data::Post for Post {
	type TagIterator<'l> = Split<'l, fn(char) -> bool>;

	fn id(&self) -> usize {
		self.id
	}

	fn md5(&self) -> &str {
		&self.hash
	}

	fn score(&self) -> isize {
		self.score
	}

	fn rating(&self) -> Rating {
		self.rating
	}

	fn resource_url(&self) -> &str {
		&self.file_url
	}

	fn tags(&self) -> Self::TagIterator<'_> {
		self.tags.split(|c| c == ' ')
	}
}
