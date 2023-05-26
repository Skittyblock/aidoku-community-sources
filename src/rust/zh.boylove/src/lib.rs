#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, Manga, MangaPageResult, Page,
};

mod parser;
use parser::{
	get_filtered_url, parse_deep_link, request_get, API_URL, BASE_URL, HTML_URL, USER_AGENT,
};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let url = get_filtered_url(filters, page);
	let json = request_get(url).json()?;

	parser::get_manga_list(json)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{}{}index/id/{}", BASE_URL, HTML_URL, id);
	let html = request_get(url).html()?;

	parser::get_manga_details(html, id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}{}chapter_list/tp/{}-0-0-10", BASE_URL, API_URL, id);
	let json = request_get(url).json()?;

	parser::get_chapter_list(json)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{}{}capter/id/{}", BASE_URL, HTML_URL, chapter_id);
	let html = request_get(url).html()?;

	parser::get_page_list(html)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
		.header("Referer", BASE_URL)
		.header("User-Agent", USER_AGENT);
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	parse_deep_link(url)
}
