#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, Filter, Listing, Manga, MangaPageResult, Page,
};

mod parser;

const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/604";
const BASE_URL: &str = "https://reaperscans.com";
const BASE_API_URL: &str = "https://api.reaperscans.com";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	parser::parse_manga_list(String::from(BASE_URL), filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	parser::parse_manga_listing(String::from(BASE_URL), listing, page)
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	parser::parse_manga_details(&String::from(BASE_URL), manga_id)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	parser::parse_chapter_list(String::from(BASE_URL), manga_id)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	parser::parse_page_list(String::from(BASE_URL), manga_id, chapter_id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	parser::modify_image_request(
		String::from(BASE_URL),
		request.header("User-Agent", USER_AGENT),
	)
}
