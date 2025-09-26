#![no_std]
#![feature(pattern)]
#![feature(iter_intersperse)]

mod constants;
mod helpers;
mod parser;
mod wrappers;

use aidoku::{
	error::Result,
	prelude::*,
	std::{String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

extern crate alloc;

#[get_manga_list]
pub fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let url = if filters.is_empty() {
		helpers::get_base_url()
	} else {
		parser::get_filter_url(&filters, page).ok_or(constants::PARSING_ERROR)?
	};
	let html = wrappers::get_html(&url)?;
	let mangas = parser::parse_manga_list(&html).ok_or(constants::PARSING_ERROR)?;
	Ok(helpers::create_manga_page_result(mangas, None))
}

#[get_manga_listing]
pub fn get_manga_listing(_listing: Listing, _page: i32) -> Result<MangaPageResult> {
	let html = wrappers::get_html(&helpers::get_base_url())?;
	let mangas = parser::parse_manga_list(&html).ok_or(constants::PARSING_ERROR)?;
	Ok(helpers::create_manga_page_result(mangas, Some(false)))
}

#[get_manga_details]
pub fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = helpers::get_manga_url(&manga_id);
	let html = wrappers::get_html(&url)?;
	parser::parse_manga(&html, manga_id).ok_or(constants::PARSING_ERROR)
}

#[get_chapter_list]
pub fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = helpers::get_manga_url(&manga_id);
	let html = wrappers::get_html(&url)?;
	parser::parse_chapters(&html, &manga_id).ok_or(constants::PARSING_ERROR)
}

#[get_page_list]
pub fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = helpers::get_chapter_url(&manga_id, &chapter_id);
	let html = wrappers::get_html(&url)?;
	parser::get_page_list(&html).ok_or(constants::PARSING_ERROR)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let manga_id = helpers::get_manga_id(&url).ok_or(constants::UNIMPLEMENTED_ERROR)?;

	Ok(DeepLink {
		manga: Some(get_manga_details(manga_id)?),
		chapter: None,
	})
}
