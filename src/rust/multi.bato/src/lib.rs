#![no_std]
#![feature(pattern)]

use aidoku::{
	error::Result, prelude::*, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

mod crypto;
mod evpkdf;
mod parser;
mod substring;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut result: Vec<Manga> = Vec::new();

	let (url, search) = parser::get_filtered_url(filters, page);
	let html = Request::new(url.as_str(), HttpMethod::Get).html();
	if search {
		parser::parse_search(html, &mut result);
	} else {
		parser::parse_listing(html, &mut result);
	}

	if !result.is_empty() {
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

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let mut url = String::from("https://bato.to");
	let mut result: Vec<Manga> = Vec::new();
	if listing.name == "Popular" {
		parser::get_list_url(&mut url, "views_a.za", page);
	}
	if listing.name == "Latest" {
		parser::get_list_url(&mut url, "update.za", page);
	}
	if listing.name == "New Titles" {
		parser::get_list_url(&mut url, "create.za", page);
	}
	let html = Request::new(url.as_str(), HttpMethod::Get).html();
	parser::parse_listing(html, &mut result);
	Ok(MangaPageResult {
		manga: result,
		has_more: true,
	})
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = format!("https://bato.to/series/{}", &manga_id);
	let html = Request::new(url.as_str(), HttpMethod::Get).html();
	parser::parse_manga(html, manga_id)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("https://bato.to/series/{}", &manga_id);
	let html = Request::new(url.as_str(), HttpMethod::Get).html();
	parser::get_chaper_list(html)
}

#[get_page_list]
fn get_page_list(chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("https://bato.to/chapter/{}", &chapter_id);
	let html = Request::new(url.as_str(), HttpMethod::Get).html();
	parser::get_page_list(html)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let parsed_manga_id = parser::parse_incoming_url(url);

	Ok(DeepLink {
		manga: Some(get_manga_details(parsed_manga_id)?),
		chapter: None,
	})
}
