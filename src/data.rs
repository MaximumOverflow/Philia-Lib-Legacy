use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Post {
	pub id: usize,
	pub tags: Tags,
	pub score: isize,
	pub rating: Rating,
	pub hash: Option<String>,
	pub preview_url: Option<String>,
	pub resource_url: Option<String>,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Rating {
	General,
	Safe,
	Sensitive,
	Questionable,
	Explicit,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Tag {
	pub id: usize,
	pub name: String,
	pub count: usize,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Tags {
	All(Vec<String>),
	Categorized(HashMap<String, Vec<String>>),
}

impl Tags {
	pub fn iter<'l>(&'l self) -> Box<dyn Iterator<Item = &'l str> + 'l> {
		match self {
			Tags::All(tags) => Box::new(tags.iter().map(|t| t.as_str())),
			Tags::Categorized(tags) => Box::new(tags.values().flat_map(|t| t.iter().map(|t| t.as_str()))),
		}
	}
}
