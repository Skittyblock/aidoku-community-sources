#![no_std]
use aidoku::{
	prelude::*, error::Result, std::String, std::Vec, std::net::Request,
	Filter, Listing, Manga, MangaPageResult, Page, Chapter, DeepLink,
};

// use mangabox_template::helper::*;
use mangabox_template::template;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	template::get_manga_list(
		String::from("https://m.mangabat.com"), 
		String::from("div.list-story-item"),
		filters, page
	)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	template::get_manga_listing(
		String::from("https://m.mangabat.com"), 
		String::from("div.list-story-item"),
		listing, page
	)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	template::get_manga_details(id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	template::get_chapter_list(id, String::from("MMM dd,yyyy HH:mm"))
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	template::get_page_list(id)
}

#[modify_image_request] 
fn modify_image_request(request: Request) {
	template::modify_image_request(
		String::from("https://m.mangabat.com"),
		request
	)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	template::handle_url(url)
}
