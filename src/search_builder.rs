use crate::source::SearchOrder;
use std::collections::HashSet;
use std::error::Error;
use crate::client::Client;
use crate::data::Post;

#[derive(Debug)]
pub struct SearchBuilder<'l> {
	client: &'l Client,

	page: u32,
	limit: u32,
	order: SearchOrder,
	include: HashSet<String>,
	exclude: HashSet<String>,
}

impl<'l> SearchBuilder<'l> {
	pub fn new(client: &'l Client) -> Self {
		Self {
			client,
			page: 1,
			limit: 16,
			order: SearchOrder::Newest,
			include: Default::default(),
			exclude: Default::default(),
		}
	}

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

	pub fn order(&mut self, order: SearchOrder) -> &mut Self {
		self.order = order;
		self
	}

	pub fn limit(&mut self, limit: u32) -> &mut Self {
		self.limit = limit;
		self
	}

	pub fn page(&mut self, page: u32) -> &mut Self {
		self.page = page;
		self
	}

	pub fn search(self) -> Result<Vec<Post>, Box<dyn Error>> {
		self.client
			.search(self.page, self.limit, self.order, self.include.into_iter(), self.exclude.into_iter())
	}

	pub async fn search_async(self) -> Result<Vec<Post>, Box<dyn Error>> {
		self.client
			.search_async(self.page, self.limit, self.order, self.include.into_iter(), self.exclude.into_iter())
			.await
	}
}
