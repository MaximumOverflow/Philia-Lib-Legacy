use crate::{Error as SearchError, Rating, Timestamp};
use crate::search::internal_traits::Search;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Post {
	pub id: usize,
	pub created_at: Timestamp,
	pub uploader_id: usize,
	pub score: isize,
	pub source: String,
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
	pub file_url: String,
	pub large_file_url: String,
	pub preview_file_url: String,
}

impl Post {
	pub fn tags(&self) -> impl Iterator<Item = &str> {
		self.tag_string.split(|c| c == ' ')
	}

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

#[derive(Debug, Deserialize)]
pub struct Error {
	message: String,
}

impl Search for Post {
	fn search(tags: String, limit: usize) -> Result<Vec<Self>, SearchError>
	where
		Self: Sized,
	{
		use SearchError::*;
		let url = format!("https://danbooru.donmai.us/posts.json?limit={limit}&tags={tags}+-status:deleted");
		let result = reqwest::blocking::get(url).map_err(|e| RequestFailed(e))?;
		let byte_vec = result.bytes().map_err(|_| EmptyResponse).map(|b| b.to_vec())?;
		let bytes = byte_vec.as_slice();
		
		match bytes {
			//['[', .., ']'] | ['[', .., ']', '\n']
			[0x5B, .., 0x5D] | [0x5B, .., 0x5D, 0x0A] => {
				let posts = serde_json::from_slice::<Vec<Post>>(bytes)
					.map_err(|e| JsonDeserializationFailed(e))?;
				
				Ok(posts)
			},

			//['{', .., '}'] | ['{', .., '}', '\n']
			[0x7B, .., 0x7D] | [0x7B, .., 0x7D, 0x0A] => {
				let error = serde_json::from_slice::<Error>(bytes)
					.map_err(|e| JsonDeserializationFailed(e))?;
				
				Err(Generic(error.message))
			},
			
			_ => Err(InvalidResponse(byte_vec)),
		}
	}
}

impl crate::Post for Post {
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
		&self.file_url
	}

	fn tags<'l>(&'l self) -> Box<dyn Iterator<Item = &'l str> + 'l> {
		Box::new(self.tags())
	}
}
