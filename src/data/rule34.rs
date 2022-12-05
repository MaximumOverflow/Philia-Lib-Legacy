use crate::data::{GenericPost, Rating, Post as PostTrait};
use serde::Deserialize;
use std::str::Split;

#[derive(Debug, Clone, Deserialize)]
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

impl PostTrait for Post {
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

	fn resource_url(&self) -> Option<&str> {
		Some(&self.file_url)
	}

	fn tags(&self) -> Self::TagIterator<'_> {
		self.tags.split(|c| c == ' ')
	}
}

impl From<Post> for GenericPost {
	fn from(post: Post) -> Self {
		Self {
			tags: post.tags_owned(),
			id: post.id,
			md5: post.hash,
			score: post.score,
			rating: post.rating,
			resource_url: post.file_url,
		}
	}
}