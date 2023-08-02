use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Post {
	#[serde(default = "Default::default")]
	#[cfg(feature = "additional_post_metadata")]
	pub source: String,

	pub id: usize,
	pub tags: Tags,
	pub score: isize,
	pub rating: Rating,
	pub hash: Option<String>,
	pub preview_url: Option<String>,
	pub resource_url: Option<String>,

	#[serde(default = "Default::default")]
	#[serde(deserialize_with = "serde_functions::deserialize_dimensions")]
	pub preview_dimensions: Option<(u32, u32)>,

	#[serde(default = "Default::default")]
	#[serde(deserialize_with = "serde_functions::deserialize_dimensions")]
	pub resource_dimensions: Option<(u32, u32)>,
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
	All(HashSet<String>),
	Categorized(HashMap<String, HashSet<String>>),
}

impl Tags {
	pub fn iter<'l>(&'l self) -> Box<dyn Iterator<Item = &'l str> + 'l> {
		match self {
			Tags::All(tags) => Box::new(tags.iter().map(|t| t.as_str())),
			Tags::Categorized(tags) => Box::new(tags.values().flat_map(|t| t.iter().map(|t| t.as_str()))),
		}
	}
	
	pub fn contains(&self, tag: &str) -> bool {
		match self {
			Tags::All(tags) => tags.contains(tag),
			Tags::Categorized(tags) => tags.values().any(|tags| tags.contains(tag)),
		}
	}
}

mod serde_functions {
	use serde::{Deserialize, Deserializer};

	pub fn deserialize_dimensions<'de, D>(deserializer: D) -> Result<Option<(u32, u32)>, D::Error>
	where
		D: Deserializer<'de>,
	{
		let dim = Option::<(u32, u32)>::deserialize(deserializer)?;
		match dim {
			Some((0, _)) | Some((_, 0)) => Ok(None),
			_ => Ok(dim),
		}
	}
}
