#![no_std]
use aidoku::{
	error::Result, prelude::*, std::net::Request, std::String, std::Vec, Chapter, DeepLink, Filter,
	Listing, Manga, MangaPageResult, Page,
};

use mangaworld_template::template;

const BASE_URL: &str = "https://www.mangaworldadult.com";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	template::parse_manga_list(String::from(BASE_URL), filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	template::parse_manga_listing(String::from(BASE_URL), listing.name, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	template::parse_manga_details(String::from(BASE_URL), id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	template::parse_chapter_list(String::from(BASE_URL), id)
}

#[get_page_list]
fn get_page_list(manga_id: String, id: String) -> Result<Vec<Page>> {
	template::parse_page_list(String::from(BASE_URL), manga_id, id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	template::modify_image_request(String::from(BASE_URL), request)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	template::handle_url(String::from(BASE_URL), url)
}
