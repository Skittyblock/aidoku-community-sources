#![no_std]
extern crate alloc;
mod parser;
mod url;

use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};
use alloc::string::ToString;
use parser::MangaListResponse;
use url::Url;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let manga_list_url = Url::from((filters, page));
	let manga_list_request = Request::get(manga_list_url.to_string());

	if let Url::Filters { .. } = manga_list_url {
		let filters_page = manga_list_request.html()?;
		return filters_page.get_page_result();
	}

	let search_json = manga_list_request.json()?;
	search_json.get_page_result()
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	todo!()
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	todo!()
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	todo!()
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	todo!()
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	todo!()
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	todo!()
}
