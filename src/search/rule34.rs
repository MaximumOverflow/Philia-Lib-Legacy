use crate::search::{Search, SearchBuilder, SearchResult, Error, SearchAsync, SearchFuture};
use crate::rule34::{Post, Rule34};
use serde_json::Value;

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
	use crate::data::Rating::*;
	let bytes = byte_vec.as_slice();

	let data = serde_json::from_slice::<Value>(bytes)
		.map_err(|err| Error::from((bytes, err)))?;
	
	match data {
		Value::Array(values) => {
			let posts = values.iter().filter_map(|value| {
				let object = value.as_object()?;
				let post = Post {
					id: object.get("id")?.as_u64()? as usize,
					directory: object.get("directory")?.as_u64()? as usize,
					hash: object.get("hash")?.as_str()?.to_owned(),
					width: object.get("width")?.as_u64()? as usize,
					height: object.get("height")?.as_u64()? as usize,
					image: object.get("image")?.as_str()?.to_owned(),
					change: object.get("change")?.as_u64()? as usize,
					owner: object.get("owner")?.as_str()?.to_owned(),
					parent_id: object.get("parent_id")?.as_u64()? as usize,
					rating: match object.get("rating")?.as_str()? {
						"safe" => Some(Safe),
						"general" => Some(General),
						"explicit" => Some(Explicit),
						"sensitive" => Some(Sensitive),
						"questionable" => Some(Questionable),
						_ => None,
					}?,
					sample: object.get("sample")?.as_u64()? as usize,
					sample_width: object.get("sample_width")?.as_u64()? as usize,
					sample_height: object.get("sample_height")?.as_u64()? as usize,
					score: object.get("score")?.as_i64()? as isize,
					tags: object.get("tags")?.as_str()?.to_owned(),
					file_url: object.get("file_url")?.as_str()?.to_owned(),
					sample_url: object.get("sample_url")?.as_str()?.to_owned(),
					preview_url: object.get("preview_url")?.as_str()?.to_owned(),
				};
				Some(post)
			}).collect();
			
			Ok(posts)
		},
		
		_ => match String::from_utf8(byte_vec) {
			Ok(text) => Err(InvalidResponse(text)),
			Err(error) => Err(InvalidResponseBytes(error.into_bytes())),
		}
	}
}
