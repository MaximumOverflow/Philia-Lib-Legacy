use crate::search::{Error, Search, SearchAsync, SearchBuilder, SearchFuture, SearchResult};
use crate::danbooru::{Danbooru, Post};
use serde::Deserialize;

impl Search for Danbooru {
	type Post = Post;

	fn search(&self, params: SearchBuilder) -> SearchResult<Self::Post> {
		use Error::*;
		let limit = params.limit;
		let tags = params.get_joined_tags();
		let url = format!("https://danbooru.donmai.us/posts.json?limit={limit}&tags={tags}+-status:deleted");
		let result = reqwest::blocking::get(url).map_err(RequestFailed)?;
		let bytes = result.bytes().map_err(|_| EmptyResponse).map(|b| b.to_vec())?;
		deserialize(bytes)
	}
}

impl SearchAsync for Danbooru {
	type Post = Post;

	fn search_async(&self, params: SearchBuilder) -> SearchFuture<Self::Post> {
		async fn search_async(params: SearchBuilder) -> Result<Vec<Post>, Error> {
			use Error::*;
			let limit = params.limit;
			let tags = params.get_joined_tags();
			let url = format!("https://danbooru.donmai.us/posts.json?limit={limit}&tags={tags}+-status:deleted");
			let result = reqwest::get(url).await.map_err(RequestFailed)?;
			let bytes = result.bytes().await.map_err(|_| EmptyResponse).map(|b| b.to_vec())?;
			deserialize(bytes)
		}

		Box::pin(search_async(params))
	}
}

fn deserialize(byte_vec: Vec<u8>) -> Result<Vec<Post>, Error> {
	use Error::*;
	let bytes = byte_vec.as_slice();
	match bytes {
		//['[', .., ']'] | ['[', .., ']', '\n']
		[0x5B, .., 0x5D] | [0x5B, .., 0x5D, 0x0A] => {
			let posts = serde_json::from_slice::<Vec<Post>>(bytes).map_err(JsonDeserializationFailed)?;
			Ok(posts)
		}

		//['{', .., '}'] | ['{', .., '}', '\n']
		[0x7B, .., 0x7D] | [0x7B, .., 0x7D, 0x0A] => {
			#[derive(Debug, Deserialize)]
			pub struct Error {
				message: String,
			}

			let error = serde_json::from_slice::<Error>(bytes).map_err(JsonDeserializationFailed)?;
			Err(Generic(error.message))
		}

		_ => match String::from_utf8(byte_vec) {
			Ok(text) => Err(InvalidResponse(text)),
			Err(error) => Err(InvalidResponseBytes(error.into_bytes())),
		},
	}
}
