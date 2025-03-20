#![no_std]
#![feature(pattern)]

use aidoku::{
	error::Result,
	prelude::*,
	std::net::{HttpMethod, Request},
	std::{String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

mod parser;

const BASE_URL: &str = "https://fanfox.net";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let url = parser::get_filtered_url(filters, page);
	let html = Request::get(url).html()?;
	parser::parse_directory(html)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let url_query = match listing.name.as_str() {
		"Latest" => "latest",
		"Updated Rating" => "rating",
		_ => "",
	};
	let url = format!("{BASE_URL}/directory/updated/{page}.html?{url_query}");
	let html = Request::get(url).html()?;
	parser::parse_directory(html)
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = format!("{BASE_URL}/manga/{}", &manga_id);
	let html = Request::get(url).html()?;
	parser::parse_manga(html, manga_id)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("{BASE_URL}/manga/{}", &manga_id);
	let html = Request::new(url.as_str(), HttpMethod::Get)
		.header("Cookie", "isAdult=1")
		.html()?;
	parser::parse_chapters(html)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("https://m.fanfox.net/roll_manga/{}/1.html", chapter_id);
	let html = Request::new(url.as_str(), HttpMethod::Get)
		.header("Cookie", "readway=2")
		.html()?;
	parser::get_page_list(html)
}

#[modify_image_request]
pub fn modify_image_request(request: Request) {
	request.header("Referer", "https://m.fanfox.net/");
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let parsed_manga_id = parser::parse_incoming_url(url);

	Ok(DeepLink {
		manga: Some(get_manga_details(parsed_manga_id)?),
		chapter: None,
	})
}
