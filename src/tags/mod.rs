pub mod e621;

use crate::{BoxFuture, Error};
use crate::tags::internal::EnableTags;

type TagsResult<T> = Result<Vec<T>, Error>;
type TagsFuture<T> = BoxFuture<'static, TagsResult<T>>;

pub trait Tags: EnableTags {
	type Tag;
	fn get_tags(&self, limit: usize, page: usize) -> TagsResult<Self::Tag>;
}

pub trait TagsAsync: Sync + EnableTags {
	type Tag;
	fn get_tags_async(&self, limit: usize, page: usize) -> TagsFuture<Self::Tag>;
}

mod internal {
	pub trait EnableTags {}
	impl EnableTags for crate::e621::E621 {}
	// impl EnableTags for crate::rule34::Rule34 {}
	// impl EnableTags for crate::danbooru::Danbooru {}
}