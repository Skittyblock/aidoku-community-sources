#![no_std]

use aidoku::{
	error::Result, std::Vec, Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};
use madtheme_template::{source, template::MadTheme};

struct MangaXYZ {
	base_url: &'static str,
}

impl MadTheme for MangaXYZ {
	#[inline]
	fn base_url(&self) -> &'static str {
		self.base_url
	}
}

static INSTANCE: MangaXYZ = MangaXYZ {
	base_url: "https://mangaxyz.com",
};

source! { INSTANCE }
