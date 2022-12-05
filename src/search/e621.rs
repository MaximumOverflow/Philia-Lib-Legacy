use crate::search::{SearchAsync, Error, Search, SearchBuilder, SearchFuture, SearchResult};
use serde_derive::Deserialize;
use crate::e621::{E621, Post};
use crate::USER_AGENT;

impl Search for E621 {
	type Post = Post;

	fn search(&self, params: SearchBuilder) -> SearchResult<Self::Post> {
		use Error::*;

		let client = reqwest::blocking::ClientBuilder::new()
			.user_agent(USER_AGENT)
			.https_only(false)
			.build()
			.unwrap();

		let limit = params.limit;
		let tags = params.get_joined_tags();
		let url = format!("https://e621.net/posts.json?limit={limit}&tags={tags}+-status:deleted");
		let result = client.get(url).send().map_err(RequestFailed)?;
		let bytes = result.bytes().map_err(|_| EmptyResponse).map(|b| b.to_vec())?;
		deserialize(bytes)
	}
}

impl SearchAsync for E621 {
	type Post = Post;

	fn search_async(&self, params: SearchBuilder) -> SearchFuture<Self::Post> {
		async fn search_async(params: SearchBuilder) -> Result<Vec<Post>, Error> {
			use Error::*;

			let client = reqwest::ClientBuilder::new().user_agent(USER_AGENT).https_only(false).build().unwrap();

			let limit = params.limit;
			let tags = params.get_joined_tags();
			let url = format!("https://e621.net/posts.json?limit={limit}&tags={tags}+-status:deleted");
			let result = client.get(url).send().await.map_err(RequestFailed)?;
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
