#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

use otaku_sanctuary_template::template::OtakuSanctuarySource;

static INSTANCE: OtakuSanctuarySource = OtakuSanctuarySource {
	base_url: "https://myrockmanga.com",
};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	INSTANCE.get_manga_list(filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	INSTANCE.get_manga_listing(listing, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	INSTANCE.get_manga_details(id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	INSTANCE.get_chapter_list(id)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	INSTANCE.get_page_list(id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	INSTANCE.modify_image_request(request)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	INSTANCE.handle_url(url)
}
