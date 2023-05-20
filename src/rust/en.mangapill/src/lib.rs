#![no_std]
use aidoku::{
	error::Result, prelude::*, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Chapter, DeepLink, Filter, Manga, MangaPageResult, Page,
};

mod parser;
use parser::{BASE_URL, USER_AGENT};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut result: Vec<Manga> = Vec::new();

	let mut url = String::new();
	parser::get_filtered_url(filters, page, &mut url);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;

	if url.contains("search") {
		parser::parse_search(html, &mut result);
	} else {
		parser::parse_recents(html, &mut result);
	}

	if result.len() >= 50 {
		Ok(MangaPageResult {
			manga: result,
			has_more: true,
		})
	} else {
		Ok(MangaPageResult {
			manga: result,
			has_more: false,
		})
	}
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = format!("https://www.mangapill.com{}", &manga_id);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	parser::parse_manga(html, manga_id)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("https://www.mangapill.com{}", &manga_id);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	parser::get_chaper_list(html)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("https://www.mangapill.com{}", &chapter_id);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	parser::get_page_list(html)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
		.header("Referer", BASE_URL)
		.header("User-Agent", USER_AGENT);
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let parsed_manga_id = parser::parse_incoming_url(url);

	Ok(DeepLink {
		manga: Some(get_manga_details(parsed_manga_id)?),
		chapter: None,
	})
}
