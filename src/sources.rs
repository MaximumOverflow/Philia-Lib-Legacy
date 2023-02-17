use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::data::Rating;
use crate::prelude::Order;
use crate::source::*;

lazy_static! {
    pub static ref RULE34: Source = Source {
		name: "Rule34".into(),

		search: search::Schema {
			base_url: "https://api.rule34.xxx/index.php?page=dapi&s=post&q=index&json=1&".into(),
			result_key: None,

			parameters: search::ParameterSchema {
				tags: "tags".into(),
				page: "pid".into(),
				limit: "limit".into(),
			},

			tag_separator: '+',
			tag_exclusion_prefix: '-',

			post: search::PostSchema {
				id: "id".into(),

				tags: search::TagSchema::All {
					key: "tags".into(),
					separator: Some(' '),
				},

				hash: "hash".into(),
				score: "score".into(),
				resource_url: "file_url".into(),

				rating: "rating".into(),
				rating_map: HashMap::from([
					("general".into(), Rating::General),
					("safe".into(), Rating::Safe),
					("sensitive".into(), Rating::Sensitive),
					("questionable".into(), Rating::Questionable),
					("explicit".into(), Rating::Explicit),
				])
			},
			
			order: HashMap::from([
				(Order::Newest, "sort:id:desc".into()),
				(Order::Oldest, "sort:id:asc".into()),
				(Order::MostLiked, "sort:score:desc".into()),
				(Order::LeastLiked, "sort:score:asc".into()),
			])
		},
		
		tag_list: None
	};
	
    pub static ref E621: Source = Source {
		name: "E621".into(),

		search: search::Schema {
			base_url: "https://e621.net/posts.json?".into(),
			result_key: Some("posts".into()),
			
			parameters: search::ParameterSchema {
				tags: "tags".into(),
				page: "page".into(),
				limit: "limit".into(),
			},

			tag_separator: '+',
			tag_exclusion_prefix: '-',

			post: search::PostSchema {
				id: "id".into(),

				tags: search::TagSchema::Categorized {
					prefix: "tags.".into(),
					categories: vec![
						"general".into(),
						"species".into(),
						"character".into(),
						"copyright".into(),
						"artist".into(),
						"lore".into(),
						"meta".into(),
					],
					separator: None,
				},
				
				hash: "file.md5".into(),
				score: "score.total".into(),
				resource_url: "file.url".into(),

				rating: "rating".into(),
				rating_map: HashMap::from([
					("g".into(), Rating::General),
					("s".into(), Rating::Sensitive),
					("q".into(), Rating::Questionable),
					("e".into(), Rating::Explicit),
				])
			},
			
			order: HashMap::from([
				(Order::Newest, "order:id_desc".into()),
				(Order::Oldest, "order:id_asc".into()),
				(Order::MostLiked, "order:score_desc".into()),
				(Order::LeastLiked, "order:score_asc".into()),
			])
		},
		
		tag_list: Some(
			tag_list::Schema {
				base_url: "https://e621.net/tags.json?".into(),
				parameters: tag_list::ParameterSchema {
					page: "page".into(),
					limit: "limit".into(),
					search: "search[order]=count".into(),
				},
				tags: tag_list::TagSchema {
					id: "id".into(),
					name: "name".into(),
					count: "post_count".into(),
				}
			}
		),
	};
	
	pub static ref DANBOORU: Source = Source {
		name: "Danbooru".into(),
		search: search::Schema {
			base_url: "https://danbooru.donmai.us/posts.json?".into(),
			result_key: None,
			
			parameters: search::ParameterSchema {
				tags: "tags".into(),
				page: "page".into(),
				limit: "limit".into(),
			},
			
			tag_separator: '+',
			tag_exclusion_prefix: '-',
			
			post: search::PostSchema {
				id: "id".into(),
				hash: "md5".into(),
				score: "score".into(),
				resource_url: "file_url".into(),
				
				rating: "rating".into(),
				rating_map: HashMap::from([
					("g".into(), Rating::General),
					("s".into(), Rating::Sensitive),
					("q".into(), Rating::Questionable),
					("e".into(), Rating::Explicit),
				]),
				
				tags: search::TagSchema::Categorized {
					separator: Some(' '),
					prefix: "tag_string_".into(),
					categories: vec![
						"general".into(),
						"character".into(),
						"copyright".into(),
						"artist".into(),
						"meta".into(),
					],
				}
			},
			
			order: HashMap::from([
				(Order::Oldest, "order:id".into()),
				(Order::Newest, "order:id_desc".into()),
				(Order::MostLiked, "order:score".into()),
				(Order::LeastLiked, "order:score_asc".into()),
			])
		},
		
		tag_list: Some(
			tag_list::Schema {
				base_url: "https://danbooru.donmai.us/tags.json?".into(),
				parameters: tag_list::ParameterSchema {
					page: "page".into(),
					limit: "limit".into(),
					search: "search[order]=count".into(),
				},
				tags: tag_list::TagSchema {
					id: "id".into(),
					name: "name".into(),
					count: "post_count".into(),
				}
			}
		),
	};
}
