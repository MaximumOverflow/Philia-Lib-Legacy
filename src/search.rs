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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
	UnimplementedSource
}

#[derive(Debug, Default)]
pub struct SearchBuilder<'l> {
	include: HashSet<&'l str>,
	exclude: HashSet<&'l str>,
}

impl <'l> SearchBuilder<'l> {
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
	
	pub fn include_tags(&mut self, tags: impl Iterator<Item=&'l str>) -> &mut Self {
		for tag in tags {
			self.include_tag(tag);
		}
		
		self
	}

	pub fn exclude_tags(&mut self, tags: impl Iterator<Item=&'l str>) -> &mut Self {
		for tag in tags {
			self.exclude_tag(tag);
		}

		self
	}
	
	pub fn search<'p>(self, source: Source) -> Result<Box<dyn Iterator<Item=&'p dyn Post>>, Error> {
		use Error::*;
		
		match source {
			_ => Err(UnimplementedSource)
		}
	}
}

struct PostIterator<'l, P: Post> {
	index: usize,
	posts: Vec<P>,
	ph: PhantomData<&'l()>,
}

impl <'l, P: 'l + Post> Iterator for PostIterator<'l, P> {
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