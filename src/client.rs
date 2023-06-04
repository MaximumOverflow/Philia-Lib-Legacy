use crate::source::{search, Source, search::Order};
use crate::data::{Post, Tag, Tags};
use std::collections::HashMap;
use serde_json::{Map, Value};

pub const DEFAULT_USER_AGENT: &str = const_format::formatcp!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone)]
pub struct Client {
	source: Source,
	user_agent: Option<String>,
}

impl Client {
	pub fn new(source: Source) -> Self {
		Self { source, user_agent: None }
	}

	pub fn with_user_agent(source: Source, agent: String) -> Self {
		Self {
			source,
			user_agent: Some(agent),
		}
	}

	pub fn source(&self) -> &Source {
		&self.source
	}

	pub fn user_agent(&self) -> &Option<String> {
		&self.user_agent
	}

	pub fn search(
		&self,
		page: usize,
		limit: usize,
		order: Order,
		include: impl Iterator<Item = String>,
		exclude: impl Iterator<Item = String>,
	) -> Result<Vec<Post>, String> {
		let search_url = self.get_search_url(page, limit, order, include, exclude);
		let result = self.make_request(search_url)?;
		self.parse_search_results(result)
	}

	pub async fn search_async(
		&self,
		page: usize,
		limit: usize,
		order: Order,
		include: impl Iterator<Item = String>,
		exclude: impl Iterator<Item = String>,
	) -> Result<Vec<Post>, String> {
		let search_url = self.get_search_url(page, limit, order, include, exclude);
		let result = self.make_async_request(search_url).await?;
		self.parse_search_results(result)
	}

	pub fn download(&self, post: &Post) -> Result<Vec<u8>, reqwest::Error> {
		let client = self.make_http_client()?;
		let url = post.resource_url.clone().unwrap_or_default();
		let result = client.get(url).send()?;
		let bytes = result.bytes()?;
		Ok(bytes.to_vec())
	}

	pub async fn download_async(&self, post: &Post) -> Result<Vec<u8>, reqwest::Error> {
		let client = self.make_async_http_client()?;
		let url = post.resource_url.clone().unwrap_or_default();
		let result = client.get(url).send().await?;
		let bytes = result.bytes().await?;
		Ok(bytes.to_vec())
	}

	pub fn tags(&self, page: usize, limit: usize) -> Result<Vec<Tag>, String> {
		let Some(schema) = &self.source.tag_list else {
			return Err("Unsupported operation".into())
		};

		let url = format! {
			"{}{}={}&{}={}&{}",
			schema.base_url,
			schema.parameters.page, page,
			schema.parameters.limit, limit,
			schema.parameters.search,
		};

		let result = self.make_request(url)?;
		self.parse_tag_list_results(result, &schema.result_key)
	}

	pub async fn tags_async(&self, page: usize, limit: usize) -> Result<Vec<Tag>, String> {
		let Some(schema) = &self.source.tag_list else {
			return Err("Unsupported operation".into())
		};

		let url = format! {
			"{}{}={}&{}={}&{}",
			schema.base_url,
			schema.parameters.page, page,
			schema.parameters.limit, limit,
			schema.parameters.search,
		};

		let result = self.make_async_request(url).await?;
		self.parse_tag_list_results(result, &schema.result_key)
	}

	fn make_http_client(&self) -> Result<reqwest::blocking::Client, reqwest::Error> {
		reqwest::blocking::ClientBuilder::new()
			.user_agent(match &self.user_agent {
				None => DEFAULT_USER_AGENT,
				Some(agent) => agent.as_str(),
			})
			.https_only(false)
			.build()
	}

	fn make_async_http_client(&self) -> Result<reqwest::Client, reqwest::Error> {
		reqwest::ClientBuilder::new()
			.user_agent(match &self.user_agent {
				None => DEFAULT_USER_AGENT,
				Some(agent) => agent.as_str(),
			})
			.https_only(false)
			.build()
	}

	fn make_request(&self, request: String) -> Result<Value, String> {
		let client = self.make_http_client().map_err(|e| e.to_string())?;
		let result = client.get(request).send().map_err(|e| e.to_string())?;
		let bytes = result.bytes().map_err(|e| e.to_string())?.to_vec();
		serde_json::from_slice::<Value>(&bytes).map_err(|e| e.to_string())
	}

	async fn make_async_request(&self, request: String) -> Result<Value, String> {
		let client = self.make_async_http_client().map_err(|e| e.to_string())?;
		let result = client.get(request).send().await.map_err(|e| e.to_string())?;
		let bytes = result.bytes().await.map_err(|e| e.to_string())?.to_vec();
		serde_json::from_slice::<Value>(&bytes).map_err(|e| e.to_string())
	}

	#[inline(never)]
	fn get_search_url(
		&self,
		page: usize,
		limit: usize,
		order: Order,
		include: impl Iterator<Item = String>,
		exclude: impl Iterator<Item = String>,
	) -> String {
		let tags = {
			let mut tags = String::new();

			if let Some(order) = self.source.search.order.get(&order) {
				tags.push_str(order);
			}

			for tag in include {
				tags.push('+');
				tags.push_str(&tag);
			}

			for tag in exclude {
				tags.push('+');
				tags.push('-');
				tags.push_str(&tag);
			}

			tags
		};

		let mut search_url = self.source.search.base_url.clone();
		let params = &self.source.search.parameters;
		search_url.push_str(&format!("{}={}", params.page, page));
		search_url.push_str(&format!("&{}={}", params.limit, limit));
		search_url.push_str(&format!("&{}={}", params.tags, tags));
		search_url
	}

	#[inline(never)]
	fn parse_search_results(&self, value: Value) -> Result<Vec<Post>, String> {
		let results = match &self.source.search.result_key {
			Some(key) => {
				let obj = value.as_object().ok_or("Response is not a json object.")?;

				let Some(Value::Array(posts)) = obj.get(key) else {
					return Err(format!("Could not find json array {key}."))
				};
				posts.to_vec()
			}

			None => value.as_array().ok_or("Response is not a json array.")?.to_vec(),
		};

		let key_mappings = &self.source.search.post;
		let mut posts: Vec<Post> = Vec::with_capacity(results.len());
		for post in results {
			let Value::Object(info) = post else {
				continue;
			};

			posts.push(Post {
				id: {
					let value = get_value(&info, &key_mappings.id).ok_or_else(|| format!("Missing value {}", key_mappings.id))?;

					if value.is_null() {
						continue;
					}

					value.as_i64().ok_or_else(|| format!("{} is not an i64", key_mappings.id))? as _
				},

				score: {
					let value = get_value(&info, &key_mappings.score).ok_or_else(|| format!("Missing value {}", key_mappings.score))?;

					if value.is_null() {
						continue;
					}

					value.as_i64().ok_or_else(|| format!("{} is not an i64", key_mappings.id))? as _
				},

				rating: {
					let key = get_value(&info, &key_mappings.rating).ok_or_else(|| format!("Missing value {}", key_mappings.rating))?;

					if key.is_null() {
						continue;
					}

					let key = key.as_str().ok_or_else(|| format!("{} is not a string", key_mappings.rating))?;

					*key_mappings.rating_map.get(key).ok_or_else(|| format!("Unknown rating alias '{key}'"))?
				},

				tags: {
					let tag_separator = match &key_mappings.tags {
						search::TagSchema::All { separator, .. } => *separator,
						search::TagSchema::Categorized { separator, .. } => *separator,
					};

					match tag_separator {
						None => match &key_mappings.tags {
							search::TagSchema::All { key, .. } => {
								let tags = get_value(&info, key).ok_or_else(|| format!("Missing value {}", key))?;

								let Value::Array(tags) = tags else {
									return Err(format!("{} is not a json array.", key));
								};

								let tags = tags.iter().filter_map(|tag| tag.as_str()).map(|tag| tag.to_string()).collect();

								Tags::All(tags)
							}

							search::TagSchema::Categorized { prefix, categories, .. } => {
								let mut all_tags = HashMap::with_capacity(categories.len());

								for category in categories {
									let key = format!("{prefix}{category}");
									let tags = get_value(&info, &key).ok_or_else(|| format!("Missing value {}", key))?;

									let Value::Array(tags) = tags else {
										return Err(format!("{} is not a json array.", key));
									};

									let tags = tags.iter().filter_map(|tag| tag.as_str()).map(|tag| tag.to_string()).collect();

									all_tags.insert(category.clone(), tags);
								}

								Tags::Categorized(all_tags)
							}
						},

						Some(separator) => match &key_mappings.tags {
							search::TagSchema::All { key, .. } => {
								let tags = get_value(&info, key).ok_or_else(|| format!("Missing value {}", key))?;

								let Value::String(tags) = tags else {
									return Err(format!("{} is not a string.", key));
								};

								let tags = tags.split(separator).map(|tag| tag.to_string()).collect();

								Tags::All(tags)
							}

							search::TagSchema::Categorized { prefix, categories, .. } => {
								let mut all_tags = HashMap::with_capacity(categories.len());

								for category in categories {
									let key = format!("{prefix}{category}");
									let tags = get_value(&info, &key).ok_or_else(|| format!("Missing value {}", key))?;

									let Value::String(tags) = tags else {
										return Err(format!("{} is not a string.", key));
									};

									let tags = tags.split(separator).map(|tag| tag.to_string()).collect();

									all_tags.insert(category.clone(), tags);
								}

								Tags::Categorized(all_tags)
							}
						},
					}
				},

				hash: match get_value(&info, &key_mappings.hash) {
					None => None,
					Some(value) => value.as_str().map(|v| v.to_string()),
				},

				resource_url: match get_value(&info, &key_mappings.resource_url) {
					None => None,
					Some(value) => value.as_str().map(|v| v.to_string()),
				},

				preview_url: match &key_mappings.preview_url {
					None => None,
					Some(preview_url) => match get_value(&info, preview_url) {
						None => None,
						Some(value) => value.as_str().map(|v| v.to_string()),
					}
				},
			})
		}

		Ok(posts)
	}

	fn parse_tag_list_results(&self, value: Value, key: &Option<String>) -> Result<Vec<Tag>, String> {
		let tags_array = match key {
			None => {
				let Value::Array(value) = value else {
					return Err("Value is not a json array.".into())
				};

				value.to_vec()
			}
			Some(key) => {
				let Value::Object(value) = value else {
					return Err("Value is not a json object.".into())
				};

				let Some(Value::Array(value)) = value.get(key) else {
					return Err(format!("Could not find json array {key}."))
				};
				value.to_vec()
			}
		};

		let schema = &self.source.tag_list.as_ref().unwrap().tags;
		let mut tags = Vec::with_capacity(tags_array.len());

		for info in tags_array {
			let Value::Object(info) = info else {
				continue;
			};

			tags.push(Tag {
				id: get_value(&info, &schema.id)
					.ok_or_else(|| format!("Missing value {}", schema.id))?
					.as_i64()
					.ok_or_else(|| format!("{} is not an i64", schema.id))? as _,

				count: get_value(&info, &schema.count)
					.ok_or_else(|| format!("Missing value {}", schema.count))?
					.as_i64()
					.ok_or_else(|| format!("{} is not an i64", schema.count))? as _,

				name: get_value(&info, &schema.name)
					.ok_or_else(|| format!("Missing value {}", &schema.name))?
					.as_str()
					.ok_or_else(|| format!("{} is not a string", &schema.name))?
					.to_string(),
			})
		}

		Ok(tags)
	}
}

fn get_value<'l>(info: &'l Map<String, Value>, key: &str) -> Option<&'l Value> {
	match key.find('.') {
		Some(split) => {
			let parent_key = &key[..split];
			let child_key = &key[split + 1..];
			match info.get(parent_key) {
				Some(Value::Object(info)) => get_value(info, child_key),
				_ => None,
			}
		}
		None => info.get(key),
	}
}
