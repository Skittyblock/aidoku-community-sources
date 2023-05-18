#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, Manga, MangaPageResult, Page,
};

mod parser;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let url = parser::get_filtered_url(filters, page);
	let html = parser::request_get(url).html()?;

	parser::get_manga_list(html, page)
}

#[get_manga_details]
fn get_manga_details(_id: String) -> Result<Manga> {
	todo!()
}

#[get_chapter_list]
fn get_chapter_list(_id: String) -> Result<Vec<Chapter>> {
	todo!()
}

#[get_page_list]
fn get_page_list(_manga_id: String, _chapter_id: String) -> Result<Vec<Page>> {
	todo!()
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
		.header("Referer", parser::BASE_URL)
		.header("User-Agent", parser::USER_AGENT);
}

#[handle_url]
fn handle_url(_url: String) -> Result<DeepLink> {
	todo!()
}
