#![no_std]
#![feature(pattern)]
#![feature(iter_intersperse)]

mod parser;
mod wrappers;

use aidoku::{
	error::Result,
	prelude::*,
	std::net::{HttpMethod, Request},
	std::{html::Node, String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

extern crate alloc;
use alloc::string::ToString;

use crate::wrappers::debug;

#[initialize]
pub fn initialize() {
	// println!(
	// 	"{}",
	// 	Node::new_fragment(b"<a href=\"foo\"><div>foo</div></a>")
	// 		.unwrap()
	// 		.outer_html()
	// 		.read()
	// );
	// debug!(
	// 	"{:?}",
	// 	WNode::new("<a href=\"foo\"><div>foo</div></a>".to_string()).attr("href")
	// );
}

#[get_manga_list]
pub fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let url_or_error = parser::get_filter_url(&filters, page);
	println!("filtered url: {:?}", url_or_error);
	let mangas = url_or_error
		.and_then(parser::new_get_request)
		.and_then(parser::parse_directory)?;
	// debug!("{:?}", mangas);
	let result = MangaPageResult {
		manga: mangas,
		has_more: false,
	};

	let mangafox_url = parser::get_filtered_url_mangafox(filters, page);
	let mangafox_html = Request::new(mangafox_url.as_str(), HttpMethod::Get)
		.html()
		.expect("");
	parser::parse_directory_mangafox(mangafox_html)
}

#[get_manga_listing]
pub fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let url_query = match listing.name.as_str() {
		"Latest" => "latest",
		"Updated Rating" => "rating",
		_ => "",
	};
	let url = format!(
		"https://fanfox.net/directory/updated/{}.html?{}",
		page.to_string(),
		url_query
	);
	let html = Request::new(url.as_str(), HttpMethod::Get)
		.html()
		.expect("");
	parser::parse_directory_mangafox(html)
}

#[get_manga_details]
pub fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = format!("https://www.fanfox.net/manga/{}", &manga_id);
	let html = Request::new(url.as_str(), HttpMethod::Get)
		.html()
		.expect("");
	parser::parse_manga_mangafox(html, manga_id)
}

#[get_chapter_list]
pub fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("https://www.fanfox.net/manga/{}", &manga_id);
	let html = Request::new(url.as_str(), HttpMethod::Get)
		.header("Cookie", "isAdult=1")
		.html()
		.expect("");
	parser::parse_chapters_mangafox(html)
}

#[get_page_list]
pub fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("https://m.fanfox.net/roll_manga/{}/1.html", chapter_id);
	let html = Request::new(url.as_str(), HttpMethod::Get)
		.header("Cookie", "readway=2")
		.html()
		.expect("");
	parser::get_page_list_mangafox(html)
}

#[modify_image_request]
pub fn modify_image_request(request: Request) {
	request.header("Referer", "https://m.fanfox.net/");
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let parsed_manga_id = parser::parse_incoming_url_mangafox(url);

	Ok(DeepLink {
		manga: Some(get_manga_details(parsed_manga_id)?),
		chapter: None,
	})
}
