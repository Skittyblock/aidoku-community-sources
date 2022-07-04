
#![no_std]
use aidoku::{
	prelude::*, error::Result, std::String, std::Vec, std::net::Request,
	Filter, Listing, Manga, MangaPageResult, Page, Chapter, DeepLink,
};

use ninemanga_template::template::NineMangaSource;

fn get_instance() -> NineMangaSource {
    NineMangaSource{
		base_url: "https://it.ninemanga.com",
		language: "it",
		completed_series: "Serie Completato",
		..Default::default()
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	get_instance().parse_manga_list(filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult>{
	let url = match listing.name.as_str() {
		"Ultime" => format!("{}/list/New-Update/", get_instance().base_url),
		"Popolare" => format!("{}/list/Hot-Book/", get_instance().base_url),
		_ => format!("{}/list/New-Book/", get_instance().base_url),
	};
    get_instance().parse_manga_listing(url, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	get_instance().parse_manga_details(id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	get_instance().parse_chapter_list(id)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	get_instance().parse_page_list(id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	get_instance().modify_image_request(request)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(url)
}