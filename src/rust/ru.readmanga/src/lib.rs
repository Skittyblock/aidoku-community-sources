#![no_std]
#![feature(pattern)]
#![feature(iter_intersperse)]

mod parser;
mod wrappers;

use aidoku::{
	error::Result,
	prelude::*,
	std::net::Request,
	std::{String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

extern crate alloc;
use alloc::vec;

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
	let search_url = parser::get_filter_url(&filters, parser::Sorting::default(), page)?;
	debug!("search url: {}", search_url);
	let request = parser::get_html(search_url)?;
	let mangas = parser::parse_directory(request)?;
	let result = parser::create_manga_page_result(mangas);
	debug!("get_manga_list: {result:?}");

	Ok(result)
}

#[get_manga_listing]
pub fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let sorting = parser::Sorting::from_listing(&listing);
	let url = parser::get_filter_url(&vec![], sorting, page)?;
	let html = parser::get_html(url)?;
	let mangas = parser::parse_directory(html)?;
	let result = parser::create_manga_page_result(mangas);
	debug!("get_manga_listing: {result:?}");

	Ok(result)
}

#[get_manga_details]
pub fn get_manga_details(manga_id: String) -> Result<Manga> {
	let html = parser::get_html(parser::get_manga_url(&manga_id))?;
	let result = parser::parse_manga(html, manga_id);
	debug!("get_manga_details: {result:?}");

	result
}

#[get_chapter_list]
pub fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let html = parser::get_html(parser::get_manga_url(&manga_id))?;
	let result = parser::parse_chapters(html, manga_id);
	debug!("get_chapter_list: {result:?}");

	result
}

#[get_page_list]
pub fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = parser::get_chapter_url(&_manga_id, &chapter_id);
	let html = parser::get_html(url)?;
	let result = parser::get_page_list(html);
	debug!("get_page_list: {result:?}");

	result
}

#[modify_image_request]
pub fn modify_image_request(_: Request) {}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let result = parser::parse_incoming_url(url);
	debug!("handle_url: {result:?}");

	result
}
