#![no_std]
use aidoku::{
	error::Result, prelude::*, std::net::Request, std::String, std::Vec, Chapter, DeepLink, Filter,
	Listing, Manga, MangaPageResult, Page,
};

use helper::{get_base_url, get_tag_id};
use mangastream_template::template::MangaStreamSource;
pub mod helper;
fn get_instance() -> MangaStreamSource {
	MangaStreamSource {
		tagid_mapping: get_tag_id,
		base_url: get_base_url(),
		traverse_pathname: "series",
		last_page_text_2: "التالي",
		manga_details_status: ".imptdt:contains(Status) i, r:contains(الحالة) td:eq(1)",
		chapter_date_format: "MMMM dd, yyyy",
		status_options_2: ["مستمر", "مكتمل", "متوقف", "ملغي", "متروك"],
		language_2: "ar",
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
