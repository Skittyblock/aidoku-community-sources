#![no_std]
use aidoku::{
	prelude::*, error::Result, std::String, std::Vec, std::net::Request,
	Filter, Listing, Manga, MangaPageResult, Page, Chapter, DeepLink,
};

use mangastream_template::template;

const BASE_URL: &str = "https://readkomik.com";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	template::parse_manga_list(String::from(BASE_URL), String::from("manga"), filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	template::parse_manga_listing(String::from(BASE_URL),String::from("manga"), listing, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	template::parse_manga_details(id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	template::parse_chapter_list(id, String::from("MMM dd,yyyy"), String::from("en"), "en")
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	template::parse_page_list(id)
}

#[modify_image_request] 
fn modify_image_request(request: Request) {
	template::modify_image_request(String::from(BASE_URL), request)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	template::handle_url(url)
}
