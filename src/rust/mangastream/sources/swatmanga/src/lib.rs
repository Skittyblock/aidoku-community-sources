#![no_std]

use aidoku::{
	error::Result, prelude::*, std::net::Request, std::String, std::Vec, Chapter, DeepLink, Filter,
	Listing, Manga, MangaPageResult, Page,
};

use mangastream_template::template::MangaStreamSource;

fn get_instance() -> MangaStreamSource {
	MangaStreamSource {
		has_random_chapter_prefix: false,
		listing: ["الرائج", "آخر", "جَديد"],
		base_url: String::from("https://swatscans.com"),
		manga_details_title: ".infox h1",
		manga_details_author: "span:contains(المؤلف)",
		manga_details_categories: "span:contains(Category) a, span:contains(التصنيف) a",
		manga_details_cover: ".ime noscript img",
		manga_details_type: "span:contains(Type) a, span:contains(النوع) a",
		manga_details_status: "span:contains(Status) a, span:contains(الحالة) a",
		status_options: ["ongoing", "completed", "hiatus", "", ""],
		last_page_text: "التالي",
		chapter_selector: ".bxcl ul li",
		chapter_title: "span.lchx",
		alt_pages: true,
		language: "ar",
		locale: "ar_SA",
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
	get_instance().parse_page_list(format!("manga/{manga_id}/{chapter_id}"))
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	get_instance().modify_image_request(request)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(url)
}
