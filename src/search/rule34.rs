use crate::search::{Search, SearchBuilder, SearchResult, Error, SearchAsync, SearchFuture};
use crate::rule34::{Post, Rule34};

impl Search for Rule34 {
	type Post = Post;

	fn search(&self, params: SearchBuilder) -> SearchResult<Self::Post> {
		use Error::*;
		let page = params.page;
		let limit = params.limit;
		let tags = params.get_joined_tags();
		let url = format!("https://api.rule34.xxx/index.php?page=dapi&s=post&q=index&json=1&pid={page}&limit={limit}&tags={tags}");
		let result = reqwest::blocking::get(url).map_err(RequestFailed)?;
		let bytes = result.bytes().map_err(|_| EmptyResponse).map(|b| b.to_vec())?;
		deserialize(bytes)
	}
}

impl SearchAsync for Rule34 {
	type Post = Post;

	fn search_async(&self, params: SearchBuilder) -> SearchFuture<Self::Post> {
		async fn search_async(params: SearchBuilder) -> Result<Vec<Post>, Error> {
			use Error::*;
			let page = params.page;
			let limit = params.limit;
			let tags = params.get_joined_tags();
			let url = format!("https://api.rule34.xxx/index.php?page=dapi&s=post&q=index&json=1&pid={page}&limit={limit}&tags={tags}");
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
			let posts = serde_json::from_slice::<Vec<Post>>(bytes)
				.map_err(|err| Error::from((bytes, err)))?;
			
			Ok(posts)
		}

		_ => match String::from_utf8(byte_vec) {
			Ok(text) => Err(InvalidResponse(text)),
			Err(error) => Err(InvalidResponseBytes(error.into_bytes())),
		},
	}
}
