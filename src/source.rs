use serde::{Deserialize, Serialize};
use crate::data::{Post, Tag};
use std::error::Error;
use bitflags::bitflags;
use reqwest::Url;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum SearchOrder {
	Newest,
	Oldest,
	MostLiked,
	LeastLiked,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TagOrder {
	Date,
	Name,
	Count,
}

bitflags! { 
	#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FeatureFlags: u32 {
		const NONE = 0b00000000;
        const SEARCH = 0b00000001;
        const TAG_LIST = 0b00000010;
        const FULL_TAG_LIST = 0b00000100;
        const ALL = Self::SEARCH.bits() | Self::TAG_LIST.bits() | Self::FULL_TAG_LIST.bits();
    }
}

pub trait Source where Self: Send + Sync {
	fn name(&self) -> &str;
	
	fn get_search_url(
		&self,
		page: u32,
		limit: u32,
		order: SearchOrder,
		include: Vec<String>,
		exclude: Vec<String>,
	) -> Option<Url>;
	
	fn get_tag_list_url(
		&self,
		page: u32,
		limit: u32,
		order: TagOrder,
	) -> Option<Url>;
	
	fn get_full_tag_list_url(&self) -> Option<Url> {
		None
	}
	
	fn parse_search_result(&self, result: &str) -> Result<Vec<Post>, Box<dyn Error>>;

	fn parse_tag_list(&self, result: &str) -> Result<Vec<Tag>, Box<dyn Error>>;

	fn feature_flags(&self) -> FeatureFlags;
}

#[cfg(feature = "scripting")]
pub use scripting::ScriptableSource;

#[cfg(feature = "scripting")]
mod scripting {
	use reqwest::Url;
	use std::error::Error;
	use itertools::Itertools;
	use rhai::{AST, Dynamic, Engine, Scope};
	use rhai::packages::{LanguageCorePackage, Package};
	use crate::data::{Post, Tag};
	use crate::source::{FeatureFlags, Source};
	use crate::prelude::{SearchOrder, TagOrder};

	pub struct ScriptableSource {
		ast: AST,
		name: String,
		engine: Engine,
		feature_flags: FeatureFlags,
	}
	
	impl ScriptableSource {
		pub fn new(name: &str, script: &str) -> Result<Self, Box<dyn Error>> {
			let mut engine = Engine::new();
			engine.set_max_expr_depths(128, 128);
			let lang_package = LanguageCorePackage::new();
			lang_package.register_into_engine(&mut engine);
			
			let ast: AST = engine.compile(script)?;
			
			let mut feature_flags = FeatureFlags::NONE;
			for func in ast.iter_functions() {
				match func.name {
					"get_search_url" => feature_flags |= FeatureFlags::SEARCH,
					"get_tag_list_url" => feature_flags |= FeatureFlags::TAG_LIST,
					"get_full_tag_list_url" => feature_flags |= FeatureFlags::FULL_TAG_LIST,
					_ => {}
				}
			}
			
			Ok(Self { engine, ast, feature_flags, name: name.to_string() })
		}
	}
	
	impl Source for ScriptableSource {
		fn name(&self) -> &str {
			&self.name
		}

		fn get_search_url(&self, page: u32, limit: u32, order: SearchOrder, include: Vec<String>, exclude: Vec<String>) -> Option<Url> {
			let order = match order {
				SearchOrder::Newest => 0,
				SearchOrder::Oldest => 1,
				SearchOrder::MostLiked => 2,
				SearchOrder::LeastLiked => 3,
			};
			
			let include = include.into_iter().map(|t| Dynamic::from(t)).collect_vec();
			let exclude = exclude.into_iter().map(|t| Dynamic::from(t)).collect_vec();
			
			let result = self.engine.call_fn(
				&mut Scope::new(), 
				&self.ast, 
				"get_search_url",
				(page as u32, limit as u32, order, include, exclude)
			);
			
			let result: Dynamic = result.ok()?;
			let url: String = result.into_string().ok()?;
			Url::parse(&url).ok()
		}

		fn get_tag_list_url(&self, page: u32, limit: u32, order: TagOrder) -> Option<Url> {
			let order = match order {
				TagOrder::Date => 0,
				TagOrder::Name => 1,
				TagOrder::Count => 2,
			};
			
			let result: Dynamic = self.engine.call_fn(
				&mut Scope::new(),
				&self.ast,
				"get_tag_list_url",
				(page, limit, order)
			).ok()?;

			let url: String = result.into_string().ok()?;
			Url::parse(&url).ok()
		}

		fn parse_search_result(&self, data: &str) -> Result<Vec<Post>, Box<dyn Error>> {
			let result: Dynamic = self.engine.call_fn(
				&mut Scope::new(),
				&self.ast,
				"parse_search_result",
				(data.to_string(), )
			)?;
			
			Ok(rhai::serde::from_dynamic(&result)?)
		}

		fn parse_tag_list(&self, data: &str) -> Result<Vec<Tag>, Box<dyn Error>> {
			let result: Dynamic = self.engine.call_fn(
				&mut Scope::new(),
				&self.ast,
				"parse_tag_list",
				(data.to_string(), )
			)?;

			Ok(rhai::serde::from_dynamic(&result)?)
		}

		fn feature_flags(&self) -> FeatureFlags {
			self.feature_flags
		}
	}
}