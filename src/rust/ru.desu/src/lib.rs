#![no_std]
extern crate alloc;

mod constants;
mod helper;
mod parser;

use crate::helper::{create_manga_page_result, fetch_json, get_chapter_url, get_search_url};
use crate::parser::{
	parse_chapters, parse_incoming_url, parse_manga_array, parse_manga_item, parse_pages_list,
};
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};
use helper::get_manga_url;

#[link(wasm_import_module = "net")]
extern "C" {
	fn set_rate_limit(rate_limit: i32);
	fn set_rate_limit_period(period: i32);
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn initialize() {
	set_rate_limit(constants::RATE_LIMIT);
	set_rate_limit_period(constants::RATE_LIMIT_PERIOD);
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let url = get_search_url(filters, page);
	let mangas_array = fetch_json(url)?.as_array()?;

	let mangas = parse_manga_array(mangas_array, true)?;
	Ok(create_manga_page_result(mangas))
}

#[get_manga_listing]
fn get_manga_listing(_listing: Listing, page: i32) -> Result<MangaPageResult> {
	// since there is no any listings on desu we must ignore 'listing' prop
	// but there is main page where manga appears and this is actually empty search
	get_manga_list(Vec::new(), page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = get_manga_url(id.as_str());
	let data = fetch_json(url)?.as_object()?;

	parse_manga_item(data, false)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = get_manga_url(id.as_str());
	let data = fetch_json(url)?.as_object()?;

	parse_chapters(data)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = get_chapter_url(manga_id.as_str(), chapter_id.as_str());
	let data = fetch_json(url)?.as_object()?;

	parse_pages_list(data)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
		.header("User-Agent", constants::USER_AGENT)
		.header("Referer", constants::BASE_URL);
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	parse_incoming_url(url.as_str())
}
