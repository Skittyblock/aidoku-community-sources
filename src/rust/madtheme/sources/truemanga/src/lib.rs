#![no_std]

use aidoku::{
	error::Result, std::Vec, Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};
use madtheme_template::{source, template::MadTheme};

struct TrueManga {
	base_url: &'static str,
}

impl MadTheme for TrueManga {
	#[inline]
	fn base_url(&self) -> &'static str {
		self.base_url
	}
}

static INSTANCE: TrueManga = TrueManga {
	base_url: "https://truemanga.com",
};

source! { INSTANCE }
