#![no_std]
#![feature(pattern)]
#![feature(iter_intersperse)]

mod constants;
mod helpers;
mod parser;
mod sorting;
mod wrappers;

use aidoku::{
	error::Result,
	prelude::*,
	std::{String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

extern crate alloc;

use crate::sorting::Sorting;

#[get_manga_list]
pub fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let search_url = parser::get_filter_url(&filters, &Sorting::default(), page)?;
	let html = helpers::get_html(&search_url)?;
	let mangas = parser::parse_search_results(&html)?;
	Ok(helpers::create_manga_page_result(mangas))
}

#[get_manga_listing]
pub fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let sorting = Sorting::from_listing(&listing);
	let url = parser::get_filter_url(&[], &sorting, page)?;
	let html = helpers::get_html(&url)?;
	let mangas = parser::parse_search_results(&html)?;
	Ok(helpers::create_manga_page_result(mangas))
}

#[get_manga_details]
pub fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = helpers::get_manga_url(&manga_id);
	let html = helpers::get_html(&url)?;
	parser::parse_manga(&html, manga_id)
}

#[get_chapter_list]
pub fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = helpers::get_manga_url(&manga_id);
	let html = helpers::get_html(&url)?;
	parser::parse_chapters(&html, &manga_id)
}

#[get_page_list]
pub fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = helpers::get_chapter_url(&manga_id, &chapter_id);
	let html = helpers::get_html(&url)?;
	parser::get_page_list(&html)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	parser::parse_incoming_url(&url)
}
