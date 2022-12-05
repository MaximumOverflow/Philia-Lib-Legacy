use futures::future::BoxFuture;
use crate::data::Post;
use crate::USER_AGENT;

pub trait Download {
	fn download(&self) -> Result<Vec<u8>, Error>;
}

pub trait DownloadAsync {
	fn download_async(&self) -> BoxFuture<Result<Vec<u8>, Error>>;
}

#[derive(Debug)]
pub enum Error {
	MissingRemoteResource,
	ResponseError(reqwest::Error),
}

impl<T: Post> Download for T {
	fn download(&self) -> Result<Vec<u8>, Error> {
		let Some(ulr) = self.resource_url() else {
			return Err(Error::MissingRemoteResource)
		};

		let client = reqwest::blocking::ClientBuilder::new()
			.user_agent(USER_AGENT)
			.https_only(false)
			.build()
			.unwrap();

		let result = client.get(ulr).send().map_err(Error::ResponseError)?;
		let bytes = result.bytes().map_err(Error::ResponseError)?;
		Ok(bytes.to_vec())
	}
}

impl<T: Post> DownloadAsync for T {
	fn download_async(&self) -> BoxFuture<Result<Vec<u8>, Error>> {
		async fn download_async(url: Option<&str>) -> Result<Vec<u8>, Error> {
			let Some(url) = url else {
				return Err(Error::MissingRemoteResource)
			};

			let client = reqwest::ClientBuilder::new().user_agent(USER_AGENT).https_only(false).build().unwrap();
			let result = client.get(url).send().await.map_err(Error::ResponseError)?;
			let bytes = result.bytes().await.map_err(Error::ResponseError)?;
			Ok(bytes.to_vec())
		}

		Box::pin(download_async(self.resource_url()))
	}
}
