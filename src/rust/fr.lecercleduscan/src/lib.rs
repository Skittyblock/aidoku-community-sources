#![no_std]
use aidoku::{
	prelude::*,
	error::Result,
	std::{
		net::{Request,HttpMethod},
		String, Vec
	},
	Filter, FilterType, Manga, MangaPageResult, Page, Chapter, Listing
};

mod parser;
mod helper;

const BASE_URL: &str = "https://lel.lecercleduscan.com";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut url = format!("{}/directory/{}", String::from(BASE_URL), helper::i32_to_string(page));
	let mut html = Request::new(&url, HttpMethod::Get).html().expect("Failed to load HTML");	
	let mut query = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(value) = filter.value.as_string() {
					query = helper::urlencode(value.read());
				}
			},
			_ => continue,
		}
	}

	if !query.is_empty() {
		url = format!("{}/search/", String::from(BASE_URL));
		html = Request::new(url, HttpMethod::Post)
			.header("Host", "lel.lecercleduscan.com")
			.header("Content-Type", "application/x-www-form-urlencoded")
			.header("Content-Length", "11")
			.body(format!("search={}", query).as_bytes())
			.html()
			.expect("Failed to load HTML");	
	}

	parser::parse_manga_list(html)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let url = match listing.name.as_str() {
		"Dernières Sorties" => format!("{}/latest/{}", BASE_URL, page),
		_ => String::from(BASE_URL),
	};
	let html = Request::new(url, HttpMethod::Get).html().expect("Failed to load HTML");
	parser::parse_manga_list(html)
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = format!("{}/series/{}", String::from(BASE_URL), manga_id);
	let html = Request::new(url, HttpMethod::Get).html().expect("Failed to load HTML");
	parser::parse_manga_details(String::from(BASE_URL), manga_id, html)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/series/{}", String::from(BASE_URL), manga_id);
	let html = Request::new(url, HttpMethod::Post).html().expect("Failed to load HTML");
	parser::parse_chapter_list(String::from(BASE_URL), manga_id, html)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{}/read/{}/{}", String::from(BASE_URL), manga_id, chapter_id);
	let html = Request::new(url, HttpMethod::Get).html().expect("Failed to load HTML");
	parser::parse_page_list(html)
}
