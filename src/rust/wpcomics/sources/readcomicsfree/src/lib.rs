#![no_std]
pub mod helper;
use crate::helper::*;
use aidoku::{
	error::Result, prelude::*, std::net::Request, std::String, std::Vec, Chapter, DeepLink, Filter,
	FilterType, Listing, Manga, MangaPageResult, Page,
};
use wpcomics_template::{
	helper::{get_tag_id, urlencode},
	template::WPComicsSource,
};

fn get_instance() -> WPComicsSource {
	WPComicsSource {
		base_url: String::from("https://readcomicsfree.com"),
		listing_mapping: listing_map,

		manga_cell_image: "",

		manga_details_status_transformer: |str| String::from(str.trim()),

		chapter_skip_first: true,
		chapter_date_selector: "div.col-xs-3",

		manga_viewer_page_url_suffix: "/all",
		..Default::default()
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut title: String = String::new();
	let mut genre: String = String::new();
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				title = urlencode(filter.value.as_string()?.read());
			}
			_ => match filter.name.as_str() {
				"Genre" => {
					genre = get_tag_id(filter.value.as_int().unwrap_or(0));
				}
				_ => continue,
			},
		}
	}
	get_instance().get_manga_list(get_search_url(
		String::from("https://readcomicsfree.com"),
		title,
		genre,
		page,
	))
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	get_instance().get_manga_listing(listing, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	get_instance().get_manga_details(id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	get_instance().get_chapter_list(id)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	get_instance().get_page_list(chapter_id)
}

#[modify_image_request]
fn modify_image_request(_request: Request) {}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(url)
}
