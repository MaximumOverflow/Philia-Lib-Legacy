use crate::{Error as SearchError, Rating, Timestamp, USER_AGENT};
use crate::search::internal_traits::Search;
use std::collections::HashMap;
use serde_derive::Deserialize;
use std::future::Future;

#[derive(Debug, Deserialize)]
pub struct Post {
	pub id: usize,
	pub created_at: Timestamp,
	pub updated_at: Timestamp,
	pub file: File,
	pub preview: Preview,
	pub sample: Sample,
	pub score: Score,
	pub tags: Tags,
	pub locked_tags: Vec<String>,
	pub change_seq: usize,
	pub flags: Flags,
	pub rating: Rating,
	pub fav_count: usize,
	pub sources: Vec<String>,
	pub pools: Vec<usize>,
	pub relationships: Relationships,
	pub approver_id: Option<usize>,
	pub uploader_id: usize,
	pub description: String,
	pub comment_count: usize,
	pub is_favorited: bool,
	pub has_notes: bool,
	pub duration: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct File {
	pub url: String,
	pub ext: String,
	pub md5: String,
	pub size: usize,
	pub width: usize,
	pub height: usize,
}

#[derive(Debug, Deserialize)]
pub struct Preview {
	pub url: String,
	pub width: usize,
	pub height: usize,
}

#[derive(Debug, Deserialize)]
pub struct Sample {
	pub has: bool,
	pub url: String,
	pub width: usize,
	pub height: usize,
	#[serde(default = "Default::default")]
	pub alternatives: HashMap<String, Alternative>,
}

#[derive(Debug, Deserialize)]
pub struct Alternative {
	pub url: String,
	pub width: usize,
	pub height: usize,
}

#[derive(Debug, Deserialize)]
pub struct Score {
	pub up: isize,
	pub down: isize,
	pub total: isize,
}

#[derive(Debug, Deserialize)]
pub struct Tags {
	pub meta: Vec<String>,
	pub lore: Vec<String>,
	pub artist: Vec<String>,
	pub general: Vec<String>,
	pub species: Vec<String>,
	pub invalid: Vec<String>,
	pub character: Vec<String>,
	pub copyright: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Flags {
	pub pending: bool,
	pub flagged: bool,
	pub deleted: bool,
	pub note_locked: bool,
	pub status_locked: bool,
	pub rating_locked: bool,
	pub comment_disabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct Relationships {
	pub parent_id: Option<usize>,
	pub has_children: bool,
	pub has_active_children: bool,
	pub children: Vec<usize>,
}

impl Search for Post {
	fn search(tags: String, limit: usize) -> Result<Vec<Self>, SearchError>
	where
		Self: Sized,
	{
		use SearchError::*;

		let client = reqwest::blocking::ClientBuilder::new()
			.user_agent(USER_AGENT)
			.https_only(false)
			.build()
			.unwrap();

		let url = format!("https://e621.net/posts.json?limit={limit}&tags={tags}+-status:deleted");
		let result = client.get(url).send().map_err(RequestFailed)?;
		let bytes = result.bytes().map_err(|_| EmptyResponse).map(|b| b.to_vec())?;
		deserialize(bytes)
	}

	fn search_async(tags: String, limit: usize) -> Box<dyn Future<Output=Result<Vec<Self>, SearchError>>> where Self: Sized {
		Box::new(search_async(tags, limit))
	}
}

async fn search_async(tags: String, limit: usize) -> Result<Vec<Post>, SearchError> {
	use SearchError::*;

	let client = reqwest::ClientBuilder::new()
		.user_agent(USER_AGENT)
		.https_only(false)
		.build()
		.unwrap();

	let url = format!("https://e621.net/posts.json?limit={limit}&tags={tags}+-status:deleted");
	let result = client.get(url).send().await.map_err(RequestFailed)?;
	let bytes = result.bytes().await.map_err(|_| EmptyResponse).map(|b| b.to_vec())?;
	deserialize(bytes)
}

fn deserialize(byte_vec: Vec<u8>) -> Result<Vec<Post>, SearchError> {
	use SearchError::*;
	let bytes = byte_vec.as_slice();
	match bytes {
		//['{', .., '}'] | ['{', .., '}', '\n']
		[0x7B, .., 0x7D] | [0x7B, .., 0x7D, 0x0A] => {
			#[derive(Deserialize)]
			struct Posts {
				posts: Vec<Post>,
			}

			let posts = serde_json::from_slice::<Posts>(bytes).map_err(JsonDeserializationFailed)?;
			Ok(posts.posts)
		}

		_ => match String::from_utf8(byte_vec) {
			Ok(text) => Err(InvalidResponse(text)),
			Err(error) => Err(InvalidResponseBytes(error.into_bytes())),
		},
	}
}

impl crate::Post for Post {
	fn md5(&self) -> &str {
		&self.file.md5
	}

	fn score(&self) -> isize {
		self.score.total
	}

	fn rating(&self) -> Rating {
		self.rating
	}

	fn resource_url(&self) -> &str {
		&self.file.url
	}

	fn tags<'l>(&'l self) -> Box<dyn Iterator<Item = &'l str> + 'l> {
		Box::new(
			self.tags
				.meta
				.iter()
				.map(|t| t.as_str())
				.chain(self.tags.lore.iter().map(|t| t.as_str()))
				.chain(self.tags.artist.iter().map(|t| t.as_str()))
				.chain(self.tags.general.iter().map(|t| t.as_str()))
				.chain(self.tags.species.iter().map(|t| t.as_str()))
				.chain(self.tags.character.iter().map(|t| t.as_str()))
				.chain(self.tags.copyright.iter().map(|t| t.as_str())),
		)
	}
}