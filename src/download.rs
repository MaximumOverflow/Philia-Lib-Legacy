use std::future::Future;
use reqwest::Error;
use crate::Post;

pub trait Download where Self: Post {
	fn download(&self) -> Result<Vec<u8>, Error>;
	fn download_async(&self) -> Box<dyn Future<Output=Result<Vec<u8>, Error>> + '_>;
}

impl <T: Post> Download for T {
	fn download(&self) -> Result<Vec<u8>, Error> {
		let result = reqwest::blocking::get(self.resource_url())?;
		let bytes = result.bytes()?;
		Ok(bytes.to_vec())
	}

	fn download_async(&self) -> Box<dyn Future<Output=Result<Vec<u8>, Error>> + '_> {
		Box::new(download_async(self))
	}
}

async fn download_async(post: &dyn Post) -> Result<Vec<u8>, Error> {
	let result = reqwest::get(post.resource_url()).await?;
	let bytes = result.bytes().await?;
	Ok(bytes.to_vec())
}