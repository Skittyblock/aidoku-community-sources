#![no_std]

use aidoku::{
	error::Result, prelude::*, std::net::Request, std::String, std::Vec, Chapter, DeepLink, Filter,
	Listing, Manga, MangaPageResult, Page,
};

use mangastream_template::template::MangaStreamSource;
mod helper;

fn get_instance() -> MangaStreamSource {
	MangaStreamSource {
		listing: ["الرائج", "آخر", "جَديد"],
		base_url: String::from("https://swatmanga.co"),
		manga_details_title: ".infox h1",
		manga_details_author: "td:contains(Autor)+td",
		manga_details_description: ".desc",
		manga_details_categories: ".spe a",
		manga_details_cover: ".ime img",
		manga_details_type: ".spe b:contains(النوع)+a",
		last_page_text: "التالي",
		chapter_selector: ".bxcl ul li",
		chapter_title: "span.lchx",
		chapter_date_format: "yyyy-MM-dd",
		alt_pages: true,
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
	helper::parse_manga_details(&get_instance(), id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	get_instance().parse_chapter_list(id)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	helper::parse_page_list(&get_instance(), id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	get_instance().modify_image_request(request)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(url)
}
