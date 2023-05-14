#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, Manga, MangaPageResult, Page,
};

mod parser;
use parser::{API_URL, BASE_URL, CHAPTER_BASE_URL, USER_AGENT};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let url = parser::get_filtered_url(filters, page);

	if url.contains(API_URL) {
		let json = parser::request_get(url).json()?;
		return parser::parse_home_page(json);
	}
	let html = parser::request_get(url).html()?;
	parser::parse_search_page(html)
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = format!("{}/comic/{}", BASE_URL, manga_id);
	let html = parser::request_get(url).html()?;
	parser::get_manga_details(html, manga_id)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/comic/{}", BASE_URL, manga_id);
	let html = parser::request_get(url).html()?;
	parser::get_chapter_list(html, manga_id)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!(
		"{}/comic/chapter/{}/0_{}.html",
		CHAPTER_BASE_URL, manga_id, chapter_id
	);
	parser::get_page_list(url, chapter_id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
		.header("Referer", BASE_URL)
		.header("User-Agent", USER_AGENT);
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let (manga_id, chapter_id) = parser::parse_deep_link(url);

	let manga = (!manga_id.is_empty()).then(|| get_manga_details(manga_id).expect("Manga"));
	let chapter = (!chapter_id.is_empty()).then(|| Chapter {
		id: chapter_id,
		..Default::default()
	});

	Ok(DeepLink { manga, chapter })
}
