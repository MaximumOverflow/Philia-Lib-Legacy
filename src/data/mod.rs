use chrono::Utc;
use std::any::Any;
use serde_derive::Deserialize;
use crate::data::internal::EnablePost;

pub mod e621;
pub mod rule34;
pub mod danbooru;

pub type Timestamp = chrono::DateTime<Utc>;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize)]
pub enum Rating {
	#[serde(alias = "g")]
	#[serde(alias = "general")]
	General,
	#[serde(alias = "s")]
	#[serde(alias = "sensitive")]
	Sensitive,
	#[serde(alias = "q")]
	#[serde(alias = "questionable")]
	Questionable,
	#[serde(alias = "e")]
	#[serde(alias = "explicit")]
	Explicit,
}

pub trait Post: Any + EnablePost {
	type TagIterator<'l>: Iterator<Item = &'l str>;

	fn id(&self) -> usize;
	fn md5(&self) -> &str;
	fn score(&self) -> isize;
	fn rating(&self) -> Rating;
	fn resource_url(&self) -> &str;
	fn tags(&self) -> Self::TagIterator<'_>;

	fn file_name(&self) -> &str {
		let url = self.resource_url();
		let start = url.rfind(|c| c == '/').map(|i| i + 1).unwrap_or(0);
		&url[start..]
	}
}

pub trait GenericPost: Any + EnablePost {
	fn id(&self) -> usize;
	fn md5(&self) -> &str;
	fn score(&self) -> isize;
	fn rating(&self) -> Rating;
	fn file_name(&self) -> &str;
	fn resource_url(&self) -> &str;
	fn tags(&self) -> Box<dyn Iterator<Item = &str> + '_>;
}

impl<T: Post> GenericPost for T {
	fn id(&self) -> usize {
		self.id()
	}

	fn md5(&self) -> &str {
		self.md5()
	}

	fn score(&self) -> isize {
		self.score()
	}

	fn rating(&self) -> Rating {
		self.rating()
	}

	fn file_name(&self) -> &str {
		self.file_name()
	}

	fn resource_url(&self) -> &str {
		self.resource_url()
	}

	fn tags(&self) -> Box<dyn Iterator<Item = &str> + '_> {
		Box::new(self.tags())
	}
}

pub trait GenericPostCollection<'l> {
	fn iter(&'l self) -> Box<dyn Iterator<Item = &'l dyn GenericPost> + 'l>;
	fn to_box(val: Self) -> Box<dyn GenericPostCollection<'l>>
	where
		Self: Sized;
}

impl<'l, T: Post> GenericPostCollection<'l> for Vec<T> {
	fn iter(&'l self) -> Box<dyn Iterator<Item = &'l dyn GenericPost> + 'l> {
		Box::new(self.as_slice().iter().map(|p| p as &'l dyn GenericPost))
	}

	fn to_box(val: Self) -> Box<dyn GenericPostCollection<'l>> {
		Box::new(val)
	}
}

mod internal {
	pub trait EnablePost {}
	impl EnablePost for crate::e621::Post {}
	impl EnablePost for crate::rule34::Post {}
	impl EnablePost for crate::danbooru::Post {}
}
