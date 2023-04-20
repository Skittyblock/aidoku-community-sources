#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, Manga, MangaPageResult, Page,
};

mod parser;
use parser::{API_URL, BASE_URL, USER_AGENT};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut url = String::new();
	parser::get_filtered_url(filters, page, &mut url);

	let request = parser::request_get(url.clone());
	if url.contains(API_URL) {
		return parser::parse_home_page(request.json()?);
	}
	parser::parse_search_page(request.html()?)
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = format!("{}/comic/{}", BASE_URL, manga_id);
	parser::get_manga_details(parser::request_get(url).html()?, manga_id)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/comic/{}", BASE_URL, manga_id);
	parser::get_chapter_list(parser::request_get(url).html()?, manga_id)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!(
		"{}/comic/chapter/{}/0_{}.html",
		BASE_URL, manga_id, chapter_id
	);
	parser::get_page_list(parser::request_get(url).html()?)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
		.header("Referer", BASE_URL)
		.header("User-Agent", USER_AGENT);
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let deep_link = url;
	let (manga_id, chapter_id) = parser::parse_deep_link(deep_link);

	let manga = (!manga_id.is_empty()).then(|| get_manga_details(manga_id).expect("Manga"));
	let chapter = (!chapter_id.is_empty()).then(|| Chapter {
		id: chapter_id,
		..Default::default()
	});

	Ok(DeepLink { manga, chapter })
}
