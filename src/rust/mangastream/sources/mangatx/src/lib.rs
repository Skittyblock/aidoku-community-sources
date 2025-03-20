#![no_std]
use aidoku::{
	error::Result, prelude::*, std::net::Request, std::String, std::Vec, Chapter, DeepLink, Filter,
	Listing, Manga, MangaPageResult, Page,
};

use mangastream_template::template::MangaStreamSource;

fn get_instance() -> MangaStreamSource {
	MangaStreamSource {
		base_url: String::from("https://mangatx.cc"),
		traverse_pathname: "manga",
		directory_pathname: Some("manga-list"),
		manga_details_author: ".imptdt:contains(Author) a",
		chapter_date_format: "dd-MM-yyyy",
		next_page: ".pagination",
		last_page_text: "›",
		..Default::default()
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	get_instance().parse_manga_list(filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	get_instance().parse_manga_listing(get_instance().base_url, listing.name, page)
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
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let instance = get_instance();
	instance.parse_page_list(format!(
		"{}/{manga_id}/{chapter_id}",
		instance.traverse_pathname
	))
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	get_instance().modify_image_request(request)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(url)
}
