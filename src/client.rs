use std::error::Error;
use crate::source::{SearchOrder, Source, TagOrder};
use std::fmt::{Debug, Formatter};
use crate::data::{Post, Tag};
use itertools::Itertools;
use std::ops::Deref;
use std::sync::Arc;
use reqwest::Url;

pub const DEFAULT_USER_AGENT: &str = const_format::formatcp!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
const UNSUPPORTED: &str = "Unsupported operation";

#[derive(Clone)]
pub struct Client {
	source: Arc<dyn Source>,
	user_agent: Option<String>,
}

impl Client {
	pub fn new(source: impl Source + 'static) -> Self {
		Self {
			source: Arc::new(source),
			user_agent: None,
		}
	}

	pub fn with_user_agent(source: impl Source + 'static, agent: String) -> Self {
		Self {
			source: Arc::new(source),
			user_agent: Some(agent),
		}
	}

	pub fn source(&self) -> &dyn Source {
		self.source.deref()
	}

	pub fn user_agent(&self) -> &Option<String> {
		&self.user_agent
	}

	pub fn search(
		&self,
		page: u32,
		limit: u32,
		order: SearchOrder,
		include: impl IntoIterator<Item = String>,
		exclude: impl IntoIterator<Item = String>,
	) -> Result<Vec<Post>, Box<dyn Error>> {
		let (include, exclude) = (include.into_iter().collect_vec(), exclude.into_iter().collect_vec());
		let Some(url) = self.source.get_search_url(page, limit, order, include, exclude) else {
			return Err(UNSUPPORTED.into());
		};

		let result = self.make_request(url)?;
		self.source.parse_search_result(&result)
	}

	pub async fn search_async(
		&self,
		page: u32,
		limit: u32,
		order: SearchOrder,
		include: impl IntoIterator<Item = String>,
		exclude: impl IntoIterator<Item = String>,
	) -> Result<Vec<Post>, Box<dyn Error>> {
		let (include, exclude) = (include.into_iter().collect_vec(), exclude.into_iter().collect_vec());
		let Some(url) = self.source.get_search_url(page, limit, order, include, exclude) else {
			return Err(UNSUPPORTED.into());
		};

		let result = self.make_async_request(url).await?;
		self.source.parse_search_result(&result)
	}

	pub fn get_tags(&self, page: u32, limit: u32, order: TagOrder) -> Result<Vec<Tag>, Box<dyn Error>> {
		let Some(url) = self.source.get_tag_list_url(page, limit, order) else {
			return Err(UNSUPPORTED.into());
		};

		let result = self.make_request(url)?;
		self.source.parse_tag_list(&result)
	}

	pub async fn get_tags_async(&self, page: u32, limit: u32, order: TagOrder) -> Result<Vec<Tag>, Box<dyn Error>> {
		let Some(url) = self.source.get_tag_list_url(page, limit, order) else {
			return Err(UNSUPPORTED.into());
		};

		let result = self.make_async_request(url).await?;
		self.source.parse_tag_list(&result)
	}

	pub fn download(&self, post: &Post) -> Result<Vec<u8>, reqwest::Error> {
		let client = self.make_http_client()?;
		let url = post.resource_url.clone().unwrap_or_default();
		let result = client.get(url).send()?;
		let bytes = result.bytes()?;
		Ok(bytes.to_vec())
	}

	pub async fn download_async(&self, post: &Post) -> Result<Vec<u8>, reqwest::Error> {
		let client = self.make_async_http_client()?;
		let url = post.resource_url.clone().unwrap_or_default();
		let result = client.get(url).send().await?;
		let bytes = result.bytes().await?;
		Ok(bytes.to_vec())
	}

	fn make_http_client(&self) -> Result<reqwest::blocking::Client, reqwest::Error> {
		make_http_client(match &self.user_agent {
			None => DEFAULT_USER_AGENT,
			Some(agent) => agent.as_str(),
		})
	}

	fn make_async_http_client(&self) -> Result<reqwest::Client, reqwest::Error> {
		make_async_http_client(match &self.user_agent {
			None => DEFAULT_USER_AGENT,
			Some(agent) => agent.as_str(),
		})
	}

	fn make_request(&self, url: Url) -> Result<String, String> {
		let client = self.make_http_client().map_err(|e| e.to_string())?;
		let result = client.get(url).send().map_err(|e| e.to_string())?;
		let bytes = result.bytes().map_err(|e| e.to_string())?.to_vec();
		String::from_utf8(bytes).map_err(|e| e.to_string())
	}

	async fn make_async_request(&self, url: Url) -> Result<String, String> {
		let client = self.make_async_http_client().map_err(|e| e.to_string())?;
		let result = client.get(url).send().await.map_err(|e| e.to_string())?;
		let bytes = result.bytes().await.map_err(|e| e.to_string())?.to_vec();
		String::from_utf8(bytes).map_err(|e| e.to_string())
	}
}

impl Debug for Client {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let mut dbg = f.debug_struct("Client");
		dbg.field("source", &self.source.name());
		dbg.field("agent", &self.user_agent);
		dbg.finish()
	}
}

pub fn make_http_client(user_agent: &str) -> Result<reqwest::blocking::Client, reqwest::Error> {
	reqwest::blocking::ClientBuilder::new().user_agent(user_agent).https_only(false).build()
}

pub fn make_async_http_client(user_agent: &str) -> Result<reqwest::Client, reqwest::Error> {
	reqwest::ClientBuilder::new().user_agent(user_agent).https_only(false).build()
}
