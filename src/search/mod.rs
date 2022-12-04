mod e621;
mod rule34;
mod danbooru;

use std::collections::HashSet;
use futures::future::BoxFuture;
use std::ops::{Deref, DerefMut};
use crate::data::{GenericPostCollection, Post};
use crate::search::internal::EnableSearch;

#[derive(Debug)]
pub enum Error {
	EmptyResponse,
	Generic(String),
	InvalidResponse(String),
	InvalidResponseBytes(Vec<u8>),
	RequestFailed(reqwest::Error),
	JsonDeserializationFailed(serde_json::Error),
}

type SearchResult<T> = Result<Vec<T>, Error>;
type SearchFuture<T> = BoxFuture<'static, SearchResult<T>>;
type GenericSearchResult<'l> = Result<Box<dyn GenericPostCollection<'l> + 'l>, Error>;
type GenericSearchFuture<'l> = BoxFuture<'static, GenericSearchResult<'l>>;

pub trait Search: EnableSearch {
	type Post: Post;
	fn search(&self, params: SearchBuilder) -> SearchResult<Self::Post>;
}

pub trait SearchAsync: EnableSearch {
	type Post: Post;
	fn search(&self, params: SearchBuilder) -> SearchFuture<Self::Post>;
}

pub trait GenericSearch: EnableSearch {
	fn search(&self, params: SearchBuilder) -> GenericSearchResult<'_>;
}

pub trait GenericSearchAsync: EnableSearch {
	fn search(&self, params: SearchBuilder) -> GenericSearchFuture;
}

impl<T: Search> GenericSearch for T {
	fn search(&self, params: SearchBuilder) -> GenericSearchResult<'_> {
		Ok(Box::new(self.search(params)?))
	}
}

impl<T: 'static + SearchAsync + Clone + Send + Sync> GenericSearchAsync for T {
	fn search(&self, params: SearchBuilder) -> GenericSearchFuture
	where
		Self: Clone + Send + Sync,
	{
		let this = self.clone();
		Box::pin(async move {
			match this.search(params).await {
				Ok(val) => Ok(GenericPostCollection::to_box(val)),
				Err(err) => Err(err),
			}
		})
	}
}

#[derive(Debug, Default)]
pub struct SearchBuilder {
	limit: usize,
	include: HashSet<String>,
	exclude: HashSet<String>,
}

impl SearchBuilder {
	pub fn include_tag(&mut self, tag: &str) -> &mut Self {
		if !self.include.contains(tag) {
			self.exclude.remove(tag);
			self.include.insert(tag.to_owned());
		}

		self
	}

	pub fn exclude_tag(&mut self, tag: &str) -> &mut Self {
		if !self.exclude.contains(tag) {
			self.include.remove(tag);
			self.exclude.insert(tag.to_owned());
		}
		self
	}

	pub fn include_tags(&mut self, tags: impl IntoIterator<Item = impl AsRef<str>>) -> &mut Self {
		for tag in tags {
			self.include_tag(tag.as_ref());
		}

		self
	}

	pub fn exclude_tags(&mut self, tags: impl IntoIterator<Item = impl AsRef<str>>) -> &mut Self {
		for tag in tags {
			self.exclude_tag(tag.as_ref());
		}

		self
	}

	pub fn limit(&mut self, limit: usize) -> &mut Self {
		self.limit = limit;
		self
	}

	pub fn search<T: Search>(self, src: &T) -> SearchResult<T::Post> {
		src.search(self)
	}

	pub fn dyn_search(self, src: &dyn GenericSearch) -> GenericSearchResult {
		src.search(self)
	}

	pub async fn search_async<T: SearchAsync>(self, src: &T) -> SearchResult<T::Post> {
		src.search(self).await
	}

	pub async fn dyn_search_async(self, src: &dyn GenericSearchAsync) -> GenericSearchResult {
		src.search(self).await
	}

	pub(crate) fn get_joined_tags(&self) -> String {
		fn transform_tag(tag: &str) -> impl Iterator<Item = char> + '_ {
			tag.chars().flat_map(|mut c| {
				if c == ' ' {
					c = '_'
				}
				c.to_lowercase()
			})
		}

		let mut tags = String::new();
		for tag in self.include.iter() {
			if !tags.is_empty() {
				tags.push('+');
			}
			tags.extend(transform_tag(tag));
		}

		for tag in self.exclude.iter() {
			if !tags.is_empty() {
				tags.push('+');
			}
			tags.push('-');
			tags.extend(transform_tag(tag));
		}

		tags
	}
}

pub struct SearchBuilderFor<'l, T: Search> {
	source: &'l T,
	builder: SearchBuilder,
}

impl<T: Search> Deref for SearchBuilderFor<'_, T> {
	type Target = SearchBuilder;

	fn deref(&self) -> &Self::Target {
		&self.builder
	}
}

impl<T: Search> DerefMut for SearchBuilderFor<'_, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.builder
	}
}

impl<T: Search> SearchBuilderFor<'_, T> {
	pub fn search(self) -> SearchResult<T::Post> {
		self.builder.search(self.source)
	}

	pub async fn search_async(self) -> SearchResult<<T as SearchAsync>::Post>
	where
		T: SearchAsync,
	{
		self.builder.search_async(self.source).await
	}
}

pub trait BuildSearch
where
	Self: Sized + Search,
{
	fn build_search(&self) -> SearchBuilderFor<Self>;
}

impl<T: Search> BuildSearch for T {
	fn build_search(&self) -> SearchBuilderFor<Self> {
		SearchBuilderFor {
			source: self,
			builder: SearchBuilder::default(),
		}
	}
}

mod internal {
	pub trait EnableSearch {}
	impl EnableSearch for crate::e621::E621 {}
	impl EnableSearch for crate::rule34::Rule34 {}
	impl EnableSearch for crate::danbooru::Danbooru {}
}
