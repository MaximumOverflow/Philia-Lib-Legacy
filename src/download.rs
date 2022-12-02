use crate::{Post, USER_AGENT};
use std::future::Future;
use reqwest::Error;

pub trait Download
where
	Self: Post,
{
	fn download(&self) -> Result<Vec<u8>, Error>;
	fn download_async(&self) -> Box<dyn Future<Output = Result<Vec<u8>, Error>> + '_>;
}

impl<T: Post> Download for T {
	fn download(&self) -> Result<Vec<u8>, Error> {
		let client = reqwest::blocking::ClientBuilder::new()
			.user_agent(USER_AGENT)
			.https_only(false)
			.build()
			.unwrap();

		let result = client.get(self.resource_url()).send()?;
		let bytes = result.bytes()?;
		Ok(bytes.to_vec())
	}

	fn download_async(&self) -> Box<dyn Future<Output = Result<Vec<u8>, Error>> + '_> {
		Box::new(download_async(self))
	}
}

async fn download_async(post: &dyn Post) -> Result<Vec<u8>, Error> {
	let client = reqwest::ClientBuilder::new().user_agent(USER_AGENT).https_only(false).build().unwrap();

	let result = client.get(post.resource_url()).send().await?;
	let bytes = result.bytes().await?;
	Ok(bytes.to_vec())
}
