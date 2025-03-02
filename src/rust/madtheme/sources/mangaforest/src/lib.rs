#![no_std]

use aidoku::{
	error::Result, std::Vec, Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};
use madtheme_template::{source, template::MadTheme};

struct MangaForest {
	base_url: &'static str,
}

impl MadTheme for MangaForest {
	#[inline]
	fn base_url(&self) -> &'static str {
		self.base_url
	}
}

static INSTANCE: MangaForest = MangaForest {
	base_url: "https://mangaforest.me",
};

source! { INSTANCE }
