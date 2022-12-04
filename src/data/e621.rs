use crate::data::{Rating, Timestamp};
use std::collections::HashMap;
use serde_derive::Deserialize;
use std::slice::Iter;

#[derive(Debug, Deserialize)]
pub struct Post {
	pub id: usize,
	pub created_at: Timestamp,
	pub updated_at: Timestamp,
	pub file: File,
	pub preview: Preview,
	pub sample: Sample,
	pub score: Score,
	pub tags: Tags,
	pub locked_tags: Vec<String>,
	pub change_seq: usize,
	pub flags: Flags,
	pub rating: Rating,
	pub fav_count: usize,
	pub sources: Vec<String>,
	pub pools: Vec<usize>,
	pub relationships: Relationships,
	pub approver_id: Option<usize>,
	pub uploader_id: usize,
	pub description: String,
	pub comment_count: usize,
	pub is_favorited: bool,
	pub has_notes: bool,
	pub duration: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct File {
	pub url: String,
	pub ext: String,
	pub md5: String,
	pub size: usize,
	pub width: usize,
	pub height: usize,
}

#[derive(Debug, Deserialize)]
pub struct Preview {
	pub url: String,
	pub width: usize,
	pub height: usize,
}

#[derive(Debug, Deserialize)]
pub struct Sample {
	pub has: bool,
	pub url: String,
	pub width: usize,
	pub height: usize,
	#[serde(default = "Default::default")]
	pub alternatives: HashMap<String, Alternative>,
}

#[derive(Debug, Deserialize)]
pub struct Alternative {
	pub url: String,
	pub width: usize,
	pub height: usize,
}

#[derive(Debug, Deserialize)]
pub struct Score {
	pub up: isize,
	pub down: isize,
	pub total: isize,
}

#[derive(Debug, Deserialize)]
pub struct Tags {
	pub meta: Vec<String>,
	pub lore: Vec<String>,
	pub artist: Vec<String>,
	pub general: Vec<String>,
	pub species: Vec<String>,
	pub invalid: Vec<String>,
	pub character: Vec<String>,
	pub copyright: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Flags {
	pub pending: bool,
	pub flagged: bool,
	pub deleted: bool,
	pub note_locked: bool,
	pub status_locked: bool,
	pub rating_locked: bool,
	pub comment_disabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct Relationships {
	pub parent_id: Option<usize>,
	pub has_children: bool,
	pub has_active_children: bool,
	pub children: Vec<usize>,
}

impl crate::data::Post for Post {
	type TagIterator<'l> = TagIterator<'l>;

	fn id(&self) -> usize {
		self.id
	}

	fn md5(&self) -> &str {
		&self.file.md5
	}

	fn score(&self) -> isize {
		self.score.total
	}

	fn rating(&self) -> Rating {
		self.rating
	}

	fn resource_url(&self) -> &str {
		&self.file.url
	}

	fn tags(&self) -> Self::TagIterator<'_> {
		TagIterator {
			current: 0,
			iterators: [
				self.tags.meta.iter(),
				self.tags.lore.iter(),
				self.tags.artist.iter(),
				self.tags.general.iter(),
				self.tags.species.iter(),
				self.tags.character.iter(),
				self.tags.copyright.iter(),
			],
		}
	}
}

pub struct TagIterator<'l> {
	current: usize,
	iterators: [Iter<'l, String>; 7],
}

impl<'l> Iterator for TagIterator<'l> {
	type Item = &'l str;

	fn next(&mut self) -> Option<Self::Item> {
		let iter = &mut self.iterators.get_mut(self.current)?;

		if let Some(tag) = iter.next() {
			Some(tag)
		} else {
			self.current += 1;
			self.next()
		}
	}
}
