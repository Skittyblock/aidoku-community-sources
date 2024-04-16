#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::net::Request,
	std::{String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

pub mod helper;
pub mod parser;

const BASE_URL: &str = "https://comick.io";
const API_URL: &str = "https://api.comick.fun";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	parser::parse_manga_list(String::from(API_URL), filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	parser::parse_manga_listing(String::from(API_URL), listing.name, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	parser::parse_manga_details(String::from(API_URL), id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	parser::parse_chapter_list(String::from(API_URL), id)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	parser::parse_page_list(String::from(API_URL), chapter_id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	parser::modify_image_request(String::from(BASE_URL), request)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	parser::handle_url(String::from(BASE_URL), url)
}
