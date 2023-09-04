#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

#[initialize]
fn initialize() {
	todo!()
}

#[get_manga_list]
fn get_manga_list(_filters: Vec<Filter>, _page: i32) -> Result<MangaPageResult> {
	todo!()
}

#[get_manga_listing]
fn get_manga_listing(_listing: Listing, _page: i32) -> Result<MangaPageResult> {
	todo!()
}

#[get_manga_details]
fn get_manga_details(_id: String) -> Result<Manga> {
	todo!()
}

#[get_chapter_list]
fn get_chapter_list(_id: String) -> Result<Vec<Chapter>> {
	todo!()
}

#[get_page_list]
fn get_page_list(_chapter_id: String, _manga_id: String) -> Result<Vec<Page>> {
	todo!()
}

#[modify_image_request]
fn modify_image_request(_request: Request) {
	todo!()
}

#[handle_url]
fn handle_url(_url: String) -> Result<DeepLink> {
	todo!()
}
