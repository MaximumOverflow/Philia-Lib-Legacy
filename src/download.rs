use futures::future::BoxFuture;
use crate::data::Post;
use crate::USER_AGENT;
use reqwest::Error;

pub trait Download {
	fn download(&self) -> Result<Vec<u8>, Error>;
}

pub trait DownloadAsync {
	fn download_async(&self) -> BoxFuture<Result<Vec<u8>, Error>>;
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
}

impl<T: Post> DownloadAsync for T {
	fn download_async(&self) -> BoxFuture<Result<Vec<u8>, Error>> {
		async fn download_async(url: &str) -> Result<Vec<u8>, Error> {
			let client = reqwest::ClientBuilder::new().user_agent(USER_AGENT).https_only(false).build().unwrap();
			let result = client.get(url).send().await?;
			let bytes = result.bytes().await?;
			Ok(bytes.to_vec())
		}

		Box::pin(download_async(self.resource_url()))
	}
}
