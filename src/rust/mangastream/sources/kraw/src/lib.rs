#![no_std]
use aidoku::{
	error::Result, prelude::*, std::net::Request, std::String, std::Vec, Chapter, DeepLink, Filter,
	Listing, Manga, MangaPageResult, Page,
};

use mangastream_template::template::MangaStreamSource;
pub mod helper;
use helper::{get_listing_url, get_title_skip};

fn get_instance() -> MangaStreamSource {
	MangaStreamSource {
		base_url: String::from("https://kraw.org"),
		is_nsfw: true,
		next_page_2: ".pagination .next",
		manga_details_author: "tr:contains(Author) td:eq(1)",
		manga_details_categories: ".seriestugenre a",
		manga_title_trim: get_title_skip(),
		alt_pages: true,
		..Default::default()
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	helper::parse_manga_list(&get_instance(), filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	get_instance().parse_manga_listing(
		get_listing_url(&get_instance(), listing.name.clone(), page),
		listing.name,
		page,
	)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	get_instance().parse_manga_details(id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	get_instance().parse_chapter_list(id)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	get_instance().parse_page_list(id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	get_instance().modify_image_request(request)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(url)
}
