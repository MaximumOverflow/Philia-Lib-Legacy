use crate::tags::{Tags, TagsAsync, TagsFuture, TagsResult};
use crate::{Error, USER_AGENT};
use crate::e621::{E621, Tag};
use crate::Error::*;

impl Tags for E621 {
	type Tag = Tag;

	fn get_tags(&self, limit: usize, page: usize) -> TagsResult<Tag> {
		let url = format!("https://e621.net/tags.json?page={page}&limit={limit}&search[order]=count");

		let client = reqwest::blocking::ClientBuilder::new()
			.user_agent(USER_AGENT)
			.https_only(false)
			.build()
			.unwrap();

		let result = client.get(url).send().map_err(RequestFailed)?;
		let bytes = result.bytes().map_err(|_| EmptyResponse).map(|b| b.to_vec())?;
		deserialize(bytes)
	}
}

impl TagsAsync for E621 {
	type Tag = Tag;

	fn get_tags_async(&self, limit: usize, page: usize) -> TagsFuture<Self::Tag> {
		async fn get_tags_async(limit: usize, page: usize) -> TagsResult<Tag> {
			let url = format!("https://e621.net/tags.json?page={page}&limit={limit}&search[order]=count");

			let client = reqwest::ClientBuilder::new()
				.user_agent(USER_AGENT)
				.https_only(false)
				.build()
				.unwrap();

			let result = client.get(url).send().await.map_err(RequestFailed)?;
			let bytes = result.bytes().await.map_err(|_| EmptyResponse).map(|b| b.to_vec())?;
			deserialize(bytes)
		}
		
		Box::pin(get_tags_async(limit, page))
	}
}

fn deserialize(byte_vec: Vec<u8>) -> TagsResult<Tag> {
	use Error::*;
	let bytes = byte_vec.as_slice();
	match bytes {
		//['{', .., '}'] | ['{', .., '}', '\n']
		[0x5B, .., 0x5D] | [0x5B, .., 0x5D, 0x0A] => {
			let tags = serde_json::from_slice::<Vec<Tag>>(bytes)
				.map_err(|err| Error::from((bytes, err)))?;

			Ok(tags)
		}

		_ => match String::from_utf8(byte_vec) {
			Ok(text) => Err(InvalidResponse(text)),
			Err(error) => Err(InvalidResponseBytes(error.into_bytes())),
		},
	}
}
