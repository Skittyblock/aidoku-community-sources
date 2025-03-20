#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

use manga_chan_template::{helper::extract_f32_from_string, template::MangaChanSource};

static INSTANCE: MangaChanSource = MangaChanSource {
	base_url: "https://manga-chan.me",
	vol_chap_parser: |_, title| {
		let numbers = extract_f32_from_string(String::new(), title.clone());
		if numbers.len() > 1 && title.contains("Том") {
			(numbers[0], numbers[1])
		} else if !numbers.is_empty() {
			(-1.0, numbers[0])
		} else {
			(-1.0, -1.0)
		}
	},
	author_selector: "table.mangatitle tr:contains(Автор) span.translation a",
	custom_new_path: None,
};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	INSTANCE.get_manga_list(filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	INSTANCE.get_manga_listing(listing, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	INSTANCE.get_manga_details(id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	INSTANCE.get_chapter_list(id)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	INSTANCE.get_page_list(chapter_id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	INSTANCE.modify_image_request(request)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	INSTANCE.handle_url(url)
}
