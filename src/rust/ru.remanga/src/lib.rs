#![no_std]
extern crate alloc;

mod constants;
mod dto;
mod helper;
mod parser;

use crate::constants::{BASE_URL, RATE_LIMIT, RATE_LIMIT_PERIOD, USER_AGENT};
use crate::helper::{
	build_api_chapter_pages_url, build_api_filter_url, build_api_title_url, fetch_all_chapters,
	fetch_json, fetch_manga_info,
};
use crate::parser::{parse_manga_fetch_info, parse_manga_item, parse_manga_list, parse_pages};
use aidoku::error::{AidokuError, AidokuErrorKind};
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

#[link(wasm_import_module = "net")]
extern "C" {
	fn set_rate_limit(rate_limit: i32);
	fn set_rate_limit_period(period: i32);
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn initialize() {
	set_rate_limit(RATE_LIMIT);
	set_rate_limit_period(RATE_LIMIT_PERIOD);
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	build_api_filter_url(filters, page)
		.and_then(fetch_json)
		.and_then(|json| json.get("results").as_array())
		.and_then(parse_manga_list)
}

#[get_manga_listing]
fn get_manga_listing(_listing: Listing, page: i32) -> Result<MangaPageResult> {
	// there are some listings on remanga but i'm not able to find API in web
	// browser for this but they have a catalog list which is actually empty search
	// request
	get_manga_list(Vec::new(), page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	parse_manga_fetch_info(id)
		.and_then(|info| {
			if info.branches.is_empty() {
				fetch_manga_info(info.dir)
			} else {
				Ok(info)
			}
		})
		.and_then(|info| fetch_json(build_api_title_url(info.dir)))
		.and_then(parse_manga_item)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	fetch_all_chapters(id)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	fetch_json(build_api_chapter_pages_url(chapter_id)).and_then(parse_pages)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
		.header("User-Agent", USER_AGENT)
		.header("Referer", BASE_URL);
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let manga = url
		.split('/')
		.skip_while(|&s| s != "manga")
		.nth(1)
		.ok_or(AidokuError {
			reason: AidokuErrorKind::Unimplemented,
		})
		.and_then(|dir| get_manga_details(alloc::format!(":{dir}")))?;

	Ok(DeepLink {
		manga: Some(manga),
		chapter: None,
	})
}
