mod data;
mod download;
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
	pub use crate::data::{Post, GenericPost};
	pub use crate::download::{Download, DownloadAsync};
	pub use crate::search::{BuildSearch, Search, SearchAsync, GenericSearch, GenericSearchAsync};
}

pub const USER_AGENT: &str = const_format::formatcp!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
