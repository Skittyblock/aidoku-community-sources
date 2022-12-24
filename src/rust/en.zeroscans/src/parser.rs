use aidoku::{
	error::Result, std::net::Request, std::String, std::Vec, Chapter, Filter, Listing, Manga,
	MangaPageResult, Page,
};

use crate::helper::*;

pub fn parse_manga_list(
	base_url: String,
	filters: Vec<Filter>,
	page: i32,
) -> Result<MangaPageResult> {
	todo!()
}

pub fn parse_manga_listing(
	base_url: String,
	listing: Listing,
	page: i32,
) -> Result<MangaPageResult> {
	todo!()
}

pub fn parse_manga_details(base_url: String, manga_id: String) -> Result<Manga> {
	todo!()
}

pub fn parse_chapter_list(base_url: String, manga_id: String) -> Result<Vec<Chapter>> {
	todo!()
}

pub fn parse_page_list(
	base_url: String,
	manga_id: String,
	chapter_id: String,
) -> Result<Vec<Page>> {
	todo!()
}

pub fn modify_image_request(base_url: String, request: Request) {
	todo!()
}
