#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, Filter, Listing, Manga, MangaPageResult, Page,
};

mod parser;

const BASE_URL: &str = "https://omegascans.org";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	parser::parse_manga_list(String::from(BASE_URL), filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let list_query = match listing.name.as_str() {
		"Latest" => "latest",
		"Popular" => "total_views",
		"Alphabetical" => "title",
		_ => "",
	};
	let url = format!("https://api.omegascans.org/query?query_string=&series_status=All&order=desc&orderBy={}&series_type=Comic&page=1&perPage=1000&tags_ids=[]", list_query);
	parser::parse_manga_listing(String::from(BASE_URL), String::from(url), listing, page)
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	parser::parse_manga_details(String::from(BASE_URL), manga_id)
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
	parser::modify_image_request(String::from(BASE_URL), request)
}
