#![no_std]

use aidoku::{
	error::Result, prelude::*, std::defaults::defaults_get, std::net::HttpMethod,
	std::net::Request, std::String, std::Vec, Chapter, DeepLink, Filter, Listing, Manga,
	MangaPageResult, Page,
};

mod helper;
mod parser;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut result: Vec<Manga> = Vec::new();

	let (url, search) = parser::get_filtered_url(filters, page);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	if search {
		parser::parse_search(&html, &mut result);
	} else {
		parser::parse_listing(&html, &mut result);
	}

	let has_more: bool = !parser::is_last_page(html);
	Ok(MangaPageResult {
		manga: result,
		has_more,
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let mut url = String::new();
	if let Ok(url_str) = defaults_get("sourceURL")
		.expect("missing sourceURL")
		.as_string()
	{
		url.push_str(url_str.read().as_str());
	}
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
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	parser::parse_listing(&html, &mut result);

	let has_more: bool = !parser::is_last_page(html);
	Ok(MangaPageResult {
		manga: result,
		has_more,
	})
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let mut url = String::new();
	if let Ok(url_str) = defaults_get("sourceURL")
		.expect("missing sourceURL")
		.as_string()
	{
		url.push_str(url_str.read().as_str());
		url.push_str("/series/");
		url.push_str(&manga_id);
	}
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	parser::parse_manga(html, manga_id)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let mut url = String::new();
	if let Ok(url_str) = defaults_get("sourceURL")
		.expect("missing sourceURL")
		.as_string()
	{
		url.push_str(url_str.read().as_str());
		url.push_str("/series/");
		url.push_str(&manga_id);
	}
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	parser::get_chapter_list(html)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut url = String::new();
	if let Ok(url_str) = defaults_get("sourceURL")
		.expect("missing sourceURL")
		.as_string()
	{
		url.push_str(url_str.read().as_str());
		url.push_str("/chapter/");
		url.push_str(&chapter_id);
	}
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
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
