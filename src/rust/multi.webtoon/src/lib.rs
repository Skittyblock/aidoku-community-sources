#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::net::Request,
	std::{String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

use helper::get_base_url;

mod helper;
mod parser;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, _page: i32) -> Result<MangaPageResult> {
	parser::parse_manga_list(get_base_url(false), filters)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	parser::parse_manga_listing(get_base_url(false), listing, page)
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	parser::parse_manga_details(get_base_url(false), manga_id)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	// Using the mobile site here as it returns all chapters in one request
	parser::parse_chapter_list(get_base_url(true), manga_id)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	parser::parse_page_list(get_base_url(false), manga_id, chapter_id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	parser::modify_image_request(get_base_url(false), request)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	parser::handle_url(get_base_url(false), url)
}
