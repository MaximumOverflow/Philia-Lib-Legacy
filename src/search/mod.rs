mod e621;
mod rule34;
mod danbooru;

use crate::{BoxFuture, Error};
use std::collections::HashSet;
use crate::data::{GenericPost, Post};
use crate::search::internal::EnableSearch;

type SearchResult<T> = Result<Vec<T>, Error>;
type SearchFuture<T> = BoxFuture<'static, SearchResult<T>>;
type GenericSearchResult<'l> = Result<Vec<GenericPost>, Error>;
type GenericSearchFuture<'l> = BoxFuture<'static, GenericSearchResult<'l>>;

pub trait Search: EnableSearch {
	type Post: Post;
	fn search(&self, params: SearchBuilder) -> SearchResult<Self::Post>;
}

pub trait SearchAsync: Sync + EnableSearch {
	type Post: Post;
	fn search_async(&self, params: SearchBuilder) -> SearchFuture<Self::Post>;
}

pub trait GenericSearch: EnableSearch {
	fn search(&self, params: SearchBuilder) -> GenericSearchResult<'_>;
}

pub trait GenericSearchAsync: Sync + EnableSearch {
	fn search_async(&self, params: SearchBuilder) -> GenericSearchFuture;
}

impl<T: Search> GenericSearch for T {
	fn search(&self, params: SearchBuilder) -> GenericSearchResult<'_> {
		let posts = self.search(params)?.into_iter().map(|p| p.into());
		Ok(posts.collect())
	}
}

impl<T: 'static + SearchAsync + Clone + Send + Sync> GenericSearchAsync for T {
	fn search_async(&self, params: SearchBuilder) -> GenericSearchFuture
	where
		Self: Clone + Send + Sync,
	{
		let this = self.clone();
		Box::pin(async move {
			match this.search_async(params).await {
				Ok(posts) => {
					let posts = posts.into_iter().map(|p| p.into());
					Ok(posts.collect())
				}
				Err(err) => Err(err),
			}
		})
	}
}

#[derive(Debug, Default)]
pub struct SearchBuilder {
	page: usize,
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

	pub fn page(&mut self, page: usize) -> &mut Self {
		self.page = page;
		self
	}

	pub fn search<T: Search>(self, src: &T) -> SearchResult<T::Post> {
		src.search(self)
	}

	pub fn dyn_search(self, src: &dyn GenericSearch) -> GenericSearchResult {
		src.search(self)
	}

	pub async fn search_async<T: SearchAsync>(self, src: &T) -> SearchResult<T::Post> {
		src.search_async(self).await
	}

	pub async fn dyn_search_async(self, src: &dyn GenericSearchAsync) -> GenericSearchResult {
		src.search_async(self).await
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

impl<T: Search> SearchBuilderFor<'_, T> {
	pub fn include_tag(&mut self, tag: &str) -> &mut Self {
		self.builder.include_tag(tag);
		self
	}

	pub fn exclude_tag(&mut self, tag: &str) -> &mut Self {
		self.builder.exclude_tag(tag);
		self
	}

	pub fn include_tags(&mut self, tags: impl IntoIterator<Item = impl AsRef<str>>) -> &mut Self {
		self.builder.include_tags(tags);
		self
	}

	pub fn exclude_tags(&mut self, tags: impl IntoIterator<Item = impl AsRef<str>>) -> &mut Self {
		self.builder.exclude_tags(tags);
		self
	}

	pub fn limit(&mut self, limit: usize) -> &mut Self {
		self.builder.limit(limit);
		self
	}

	pub fn page(&mut self, page: usize) -> &mut Self {
		self.builder.page(page);
		self
	}

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
