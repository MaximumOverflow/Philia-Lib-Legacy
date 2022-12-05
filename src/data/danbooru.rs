use crate::data::{GenericPost, Rating, Timestamp, Post as PostTrait};
use serde::Deserialize;
use std::str::Split;

#[derive(Debug, Clone, Deserialize)]
pub struct Post {
	pub id: usize,
	pub created_at: Timestamp,
	pub uploader_id: usize,
	pub score: isize,
	pub source: String,
	#[serde(default = "Default::default")]
	pub md5: String,
	pub last_comment_bumped_at: Option<Timestamp>,
	pub rating: Rating,
	pub image_width: usize,
	pub image_height: usize,
	pub tag_string: String,
	pub fav_count: usize,
	pub file_ext: String,
	pub last_noted_at: Option<Timestamp>,
	pub parent_id: Option<usize>,
	pub has_children: bool,
	pub approver_id: Option<usize>,
	pub file_size: usize,
	pub up_score: isize,
	pub down_score: isize,
	pub is_pending: bool,
	pub is_flagged: bool,
	pub is_deleted: bool,
	pub tag_count: usize,
	pub updated_at: Timestamp,
	pub is_banned: bool,
	pub pixiv_id: Option<usize>,
	pub last_commented_at: Option<Timestamp>,
	pub has_active_children: bool,
	pub bit_flags: usize,
	pub has_large: bool,
	pub has_visible_children: bool,
	pub tag_string_general: String,
	pub tag_string_character: String,
	pub tag_string_copyright: String,
	pub tag_string_artist: String,
	pub tag_string_meta: String,
	#[serde(default = "Default::default")]
	pub file_url: String,
	#[serde(default = "Default::default")]
	pub large_file_url: String,
	#[serde(default = "Default::default")]
	pub preview_file_url: String,
}

impl Post {
	pub fn general_tags(&self) -> impl Iterator<Item = &str> {
		self.tag_string_general.split(|c| c == ' ')
	}

	pub fn character_tags(&self) -> impl Iterator<Item = &str> {
		self.tag_string_character.split(|c| c == ' ')
	}

	pub fn copyright_tags(&self) -> impl Iterator<Item = &str> {
		self.tag_string_copyright.split(|c| c == ' ')
	}

	pub fn artist_tags(&self) -> impl Iterator<Item = &str> {
		self.tag_string_artist.split(|c| c == ' ')
	}

	pub fn meta_tags(&self) -> impl Iterator<Item = &str> {
		self.tag_string_meta.split(|c| c == ' ')
	}
}

impl PostTrait for Post {
	type TagIterator<'l> = Split<'l, fn(char) -> bool>;

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

	fn resource_url(&self) -> Option<&str> {
		Some(&self.file_url)
	}

	fn tags(&self) -> Self::TagIterator<'_> {
		self.tag_string.split(|c| c == ' ')
	}
}

impl From<Post> for GenericPost {
	fn from(post: Post) -> Self {
		Self {
			tags: post.tags_owned(),
			id: post.id,
			md5: post.md5,
			score: post.score,
			rating: post.rating,
			resource_url: post.file_url,
		}
	}
}
