#![no_std]
extern crate alloc;

mod decoder;
mod helper;
mod parser;

use aidoku::{
	error::Result, prelude::*, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Chapter, Filter, Manga, MangaPageResult, Page,
};

const BASE_URL: &str = "https://www.manhuagui.com";
// const MOBILE_URL: &str = "https://m.manhuagui.com";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut url = String::new();

	parser::get_filtered_url(filters, page, &mut url);

	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	if url.contains("https://www.manhuagui.com/list/") {
		return parser::parse_home_page(html);
	}
	parser::parse_search_page(html)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{}/comic/{}", BASE_URL, id);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	let result = parser::parse_manga_details(html, id).unwrap();
	Ok(result)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/comic/{}", BASE_URL, id);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	let result = parser::get_chapter_list(html).unwrap();
	Ok(result)
}

#[get_page_list]
fn get_page_list(_chapter_id: String, manga_id: String) -> Result<Vec<Page>> {
	let base_url = format!("{}/comic/{}.html", BASE_URL, manga_id);

	let result = parser::get_page_list(base_url).unwrap();
	Ok(result)
}

#[modify_image_request]
pub fn modify_image_request(request: Request) {
	request.header("Referer", BASE_URL);
}
