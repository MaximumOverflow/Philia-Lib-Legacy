use std::collections::HashSet;
use std::marker::PhantomData;
use crate::Post;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Source {
	E621,
	E926,
	Rule34,
	Danbooru,
}

#[derive(Debug)]
pub enum Error {
	EmptyResponse,
	UnimplementedSource,
	
	Generic(String),
	InvalidResponse(Vec<u8>),
	RequestFailed(reqwest::Error),
	JsonDeserializationFailed(serde_json::Error),
}

#[derive(Debug, Default)]
pub struct SearchBuilder<'l> {
	limit: usize,
	include: HashSet<&'l str>,
	exclude: HashSet<&'l str>,
}

impl<'l> SearchBuilder<'l> {
	pub fn include_tag(&mut self, tag: &'l str) -> &mut Self {
		self.exclude.remove(tag);
		self.include.insert(tag);
		self
	}

	pub fn exclude_tag(&mut self, tag: &'l str) -> &mut Self {
		self.include.remove(tag);
		self.exclude.insert(tag);
		self
	}

	pub fn include_tags(&mut self, tags: impl Iterator<Item = &'l str>) -> &mut Self {
		for tag in tags {
			self.include_tag(tag);
		}

		self
	}

	pub fn exclude_tags(&mut self, tags: impl Iterator<Item = &'l str>) -> &mut Self {
		for tag in tags {
			self.exclude_tag(tag);
		}

		self
	}
	
	pub fn limit(&mut self, limit: usize) -> &mut Self {
		self.limit = limit;
		self
	}

	pub fn search_dyn<'p>(self, source: Source) -> Result<Box<dyn Iterator<Item = &'p dyn Post> + 'p>, Error> {
		use Error::*;
		use Source::*;
		use crate::search::internal_traits::Search;

		let tags = self.make_tags();

		match source {
			Danbooru => {
				let posts = crate::danbooru::Post::search(tags, self.limit)?;
				let iter = PostIterator::new(posts);
				Ok(Box::new(iter))
			}

			_ => Err(UnimplementedSource),
		}
	}

	pub fn search<T: Post>(self) -> Result<Vec<T>, Error> {
		let tags = self.make_tags();
		T::search(tags, self.limit)
	}

	fn make_tags(&self) -> String {
		let mut tags = String::new();
		for tag in self.include.iter() {
			if tags.len() != 0 {
				tags.push('+');
			}
			tags.extend(transform_tag(tag));
		}

		for tag in self.exclude.iter() {
			if tags.len() != 0 {
				tags.push('+');
			}
			tags.push('-');
			tags.extend(transform_tag(tag));
		}

		tags
	}
}

struct PostIterator<'l, T: Post> {
	index: usize,
	posts: Vec<T>,
	ph: PhantomData<&'l ()>,
}

impl<T: Post> PostIterator<'_, T> {
	fn new(posts: Vec<T>) -> Self {
		Self {
			posts,
			index: 0,
			ph: Default::default(),
		}
	}
}

impl<'l, T: 'l + Post> Iterator for PostIterator<'l, T> {
	type Item = &'l dyn Post;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index == self.posts.len() {
			None
		} else {
			let post = unsafe { &*self.posts.as_ptr().add(self.index) };
			self.index += 1;
			Some(post)
		}
	}
}

pub(crate) mod internal_traits {
	use crate::{Error, SearchBuilder};

	pub trait SearchPriv {}
	impl SearchPriv for SearchBuilder<'_> {}

	pub trait Search {
		fn search(params: String, limit: usize) -> Result<Vec<Self>, Error>
		where
			Self: Sized;
	}
}

fn transform_tag(tag: &str) -> impl Iterator<Item=char> + '_ {
	tag.chars()
		.map(|mut c| {
			if c == ' ' {
				c = '_'
			}
			c.to_lowercase()
		})
		.flatten()
}