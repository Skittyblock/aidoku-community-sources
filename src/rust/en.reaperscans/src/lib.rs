#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

mod helper;
mod parser;
mod request_helper;

const BASE_URL: &str = "https://reaperscans.com";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let (query, search) = helper::check_for_search(filters);
	if search {
		parser::parse_search(String::from(BASE_URL), query)
	} else {
		parser::parse_manga_list(String::from(BASE_URL), page)
	}
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	parser::parse_manga_listing(String::from(BASE_URL), listing, page)
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	parser::parse_manga_details(String::from(BASE_URL), manga_id)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	parser::parse_chapter_list(String::from(BASE_URL), manga_id)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	parser::parse_page_list(String::from(BASE_URL), manga_id, chapter_id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	parser::modify_image_request(String::from(BASE_URL), request)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let manga_id = helper::get_manga_id(url.clone());
	let chapter_id = helper::get_chapter_id(url);

	let manga = {
		if let Ok(manga) = parser::parse_manga_details(String::from(BASE_URL), manga_id) {
			Some(manga)
		} else {
			None
		}
	};

	let chapter = {
		if !chapter_id.is_empty() {
			Some(Chapter {
				id: chapter_id,
				..Default::default()
			})
		} else {
			None
		}
	};

	Ok(DeepLink { manga, chapter })
}
