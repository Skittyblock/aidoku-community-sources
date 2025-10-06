#![no_std]

use aidoku::{
	error::Result, std::Vec, Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};
use madtheme_template::{source, template::MadTheme};

struct MangaCute {
	base_url: &'static str,
}

impl MadTheme for MangaCute {
	#[inline]
	fn base_url(&self) -> &'static str {
		self.base_url
	}
}

static INSTANCE: MangaCute = MangaCute {
	base_url: "https://mangacute.com",
};

source! { INSTANCE }
