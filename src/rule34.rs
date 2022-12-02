use crate::search::internal_traits::Search;
use crate::{Error as SearchError, Rating};
use serde_derive::Deserialize;
use std::future::Future;

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

impl Search for Post {
	fn search(tags: String, limit: usize) -> Result<Vec<Self>, SearchError> where Self: Sized {
		use SearchError::*;
		let url = format!("https://api.rule34.xxx/index.php?page=dapi&s=post&q=index&json=1&limit={limit}&tags={tags}+-status:deleted");
		let result = reqwest::blocking::get(url).map_err(RequestFailed)?;
		let bytes = result.bytes().map_err(|_| EmptyResponse).map(|b| b.to_vec())?;
		deserialize(bytes)
	}

	fn search_async(tags: String, limit: usize) -> Box<dyn Future<Output=Result<Vec<Self>, SearchError>>> where Self: Sized {
		Box::new(search_async(tags, limit))
	}
}

async fn search_async(tags: String, limit: usize) -> Result<Vec<Post>, SearchError> {
	use SearchError::*;
	let url = format!("https://api.rule34.xxx/index.php?page=dapi&s=post&q=index&json=1&limit={limit}&tags={tags}+-status:deleted");
	let result = reqwest::get(url).await.map_err(RequestFailed)?;
	let bytes = result.bytes().await.map_err(|_| EmptyResponse).map(|b| b.to_vec())?;
	deserialize(bytes)
}

fn deserialize(byte_vec: Vec<u8>) -> Result<Vec<Post>, SearchError> {
	use SearchError::*;
	let bytes = byte_vec.as_slice();
	match bytes {
		//['[', .., ']'] | ['[', .., ']', '\n']
		[0x5B, .., 0x5D] | [0x5B, .., 0x5D, 0x0A] => {
			let posts = serde_json::from_slice::<Vec<Post>>(bytes).map_err(JsonDeserializationFailed)?;
			Ok(posts)
		}

		_ => match String::from_utf8(byte_vec) {
			Ok(text) => Err(InvalidResponse(text)),
			Err(error) => Err(InvalidResponseBytes(error.into_bytes())),
		},
	}
}

impl crate::Post for Post {
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

	fn tags<'l>(&'l self) -> Box<dyn Iterator<Item=&'l str> + 'l> {
		Box::new(self.tags.split(|c| c == ' '))
	}
}