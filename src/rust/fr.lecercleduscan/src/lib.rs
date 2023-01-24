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
	let mut html = Request::new(&url, HttpMethod::Get)
		.html()
		.expect("Failed to load HTML");	
	let mut search = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(value) = filter.value.as_string() {
					search = helper::urlencode(value.read());
				}
			},
			_ => continue,
		}
	}

	if search != "" {
		url = format!("{}/search/", String::from(BASE_URL));
		html = Request::new(url, HttpMethod::Post)
			.header("Host", "lel.lecercleduscan.com")
			.header("Content-Type", "application/x-www-form-urlencoded")
			.header("Content-Length", "11")
			.body(format!("search={}", search).as_bytes())
			.html()
			.expect("Failed to load HTML");
	}

	let manga: Vec<Manga> = parser::parse_mangas(html.clone());
	let has_more = parser::check_not_last_page(html);	
	
	Ok(MangaPageResult {
		manga,
		has_more,
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let mut manga: Vec<Manga> = Vec::new();
	let mut has_more = false;

	if listing.name == "Dernières Sorties" {
		let url = format!("{}/latest/{}", String::from(BASE_URL), helper::i32_to_string(page));

		let html = Request::new(url, HttpMethod::Get)
		.html()
		.expect("Failed to load HTML");
		
		manga = parser::parse_mangas(html.clone());
		has_more = parser::check_not_last_page(html);
	}

	Ok(MangaPageResult {
		manga,
		has_more,
	})
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = format!("{}/series/{}", String::from(BASE_URL), manga_id);
	let html = Request::new(url.clone(), HttpMethod::Get)
	.html()
	.expect("Failed to load HTML");
	return parser::parse_manga_details(String::from(BASE_URL), manga_id, html);
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/series/{}", String::from(BASE_URL), manga_id);
	let html = Request::new(url, HttpMethod::Post)
	.html()
	.expect("Failed to load HTML");
	return parser::parse_chapter_list(String::from(BASE_URL), manga_id, html);
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{}/read/{}/{}", String::from(BASE_URL), manga_id, chapter_id);
	let html = Request::new(url, HttpMethod::Get)
		.html()
		.expect("Failed to load HTML");
	return parser::parse_chapter_details(html);
}
