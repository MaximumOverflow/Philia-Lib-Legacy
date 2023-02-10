mod data;
mod download;
pub mod tags;
pub mod search;

pub mod e621 {
	#[derive(Default, Copy, Clone)]
	pub struct E621;
	pub use crate::data::e621::*;
}

pub mod rule34 {
	#[derive(Default, Copy, Clone)]
	pub struct Rule34;
	pub use crate::data::rule34::*;
}

pub mod danbooru {
	#[derive(Default, Copy, Clone)]
	pub struct Danbooru;
	pub use crate::data::danbooru::*;
}

pub mod prelude {
	pub use crate::e621;
	pub use crate::rule34;
	pub use crate::danbooru;

	pub use e621::E621;
	pub use rule34::Rule34;
	pub use danbooru::Danbooru;
	pub use crate::tags::{Tags, TagsAsync};
	pub use crate::data::{Post, GenericPost};
	pub use crate::download::{Download, DownloadAsync};
	pub use crate::search::{BuildSearch, Search, SearchAsync, GenericSearch, GenericSearchAsync};
}

type BoxFuture<'l, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'l>>;
pub const USER_AGENT: &str = const_format::formatcp!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

#[derive(Debug)]
pub enum Error {
	EmptyResponse,
	Generic(String),
	InvalidResponse(String),
	InvalidResponseBytes(Vec<u8>),
	RequestFailed(reqwest::Error),
	JsonDeserializationFailed {
		nearby_json: String,
		error: serde_json::Error,
	},
}

impl From<(&[u8], serde_json::Error)> for Error {
	fn from((bytes, error): (&[u8], serde_json::Error)) -> Self {
		let text = std::str::from_utf8(bytes).unwrap();

		let start = error.column().checked_sub(16).unwrap_or_default();
		let end = error.column().checked_add(16).unwrap_or(text.len());
		let nearby_json = &text[start..end];

		Error::JsonDeserializationFailed {
			nearby_json: nearby_json.to_string(),
			error,
		}
	}
}