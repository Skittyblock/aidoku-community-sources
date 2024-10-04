#![no_std]

use aidoku::{
	error::Result, std::Vec, Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};
use madtheme_template::{source, template::MadTheme};

struct MangaPuma {
	base_url: &'static str,
}

impl MadTheme for MangaPuma {
	#[inline]
	fn base_url(&self) -> &'static str {
		self.base_url
	}
}

static INSTANCE: MangaPuma = MangaPuma {
	base_url: "https://mangapuma.com",
};

source! { INSTANCE }
