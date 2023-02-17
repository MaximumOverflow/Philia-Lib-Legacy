use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Source {
	pub name: String,
	pub search: search::Schema,
	pub tag_list: Option<tag_list::Schema>,
}

pub mod search {
	use serde::{Deserialize, Serialize};
	use std::collections::HashMap;
	use crate::data::Rating;

	#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
	pub enum Order {
		Newest,
		Oldest,
		MostLiked,
		LeastLiked,
	}

	#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
	pub struct Schema {
		pub base_url: String,
		pub tag_separator: char,
		pub tag_exclusion_prefix: char,
		pub result_key: Option<String>,
		pub post: PostSchema,
		pub parameters: ParameterSchema,
		pub order: HashMap<Order, String>,
	}

	#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
	pub struct ParameterSchema {
		pub tags: String,
		pub page: String,
		pub limit: String,
	}

	#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
	pub struct PostSchema {
		pub id: String,
		pub hash: String,
		pub score: String,
		pub resource_url: String,

		pub tags: TagSchema,

		pub rating: String,
		pub rating_map: HashMap<String, Rating>,
	}

	#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
	pub enum TagSchema {
		All {
			key: String,
			separator: Option<char>,
		},

		Categorized {
			prefix: String,
			categories: Vec<String>,
			separator: Option<char>,
		},
	}
}

pub mod tag_list {
	use serde::{Deserialize, Serialize};

	#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
	pub struct Schema {
		pub base_url: String,
		pub result_key: Option<String>,
		pub parameters: ParameterSchema,
		pub tags: TagSchema,
	}

	#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
	pub struct ParameterSchema {
		pub page: String,
		pub limit: String,
		pub search: String,
	}

	#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
	pub struct TagSchema {
		pub id: String,
		pub name: String,
		pub count: String,
	}
}
