#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, Filter, Manga, MangaPageResult, Page,
};

mod parser;

const BASE_URL: &str = "https://www.baozimh.com";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut url = String::new();
	parser::get_filtered_url(filters, page, &mut url);

	if url.contains("https://www.baozimh.com/api/bzmhq/amp_comic_list") {
		let json_data = Request::new(url.as_str(), HttpMethod::Get).json()?;
		return parser::parse_home_page(json_data);
	}
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	parser::parse_search_page(html)
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = format!("{}/comic/{}", BASE_URL, manga_id);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	parser::get_manga_details(html, manga_id)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/comic/{}", BASE_URL, manga_id);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	parser::get_chapter_list(html, manga_id)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter: String) -> Result<Vec<Page>> {
	let url = format!("{}/comic/chapter/{}/0_{}.html", BASE_URL, manga_id, chapter);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	parser::get_page_list(html)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.header("Referer", BASE_URL);
}

// #[handle_url]
// fn handle_url(_: String) -> Result<DeepLink> {
//     todo!()
// }
