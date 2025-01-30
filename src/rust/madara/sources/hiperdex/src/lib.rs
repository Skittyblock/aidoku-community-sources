#![no_std]
use aidoku::{
	error::Result, prelude::*, std::defaults::defaults_get, std::net::Request, std::String,
	std::Vec, Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

use madara_template::template;

const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/604";

fn get_data() -> template::MadaraSiteData {
	let base_url = defaults_get("sourceURL")
		.expect("missing sourceURL")
		.as_string()
		.expect("missing sourceURL")
		.read();
	let data: template::MadaraSiteData = template::MadaraSiteData {
		base_url,
		user_agent: Some(USER_AGENT.into()),
		alt_ajax: true,
		..Default::default()
	};
	data
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	template::get_manga_list(filters, page, get_data())
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	template::get_manga_listing(get_data(), listing, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	template::get_manga_details(id, get_data())
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	template::get_chapter_list(id, get_data())
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	template::get_page_list(chapter_id, get_data())
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	template::modify_image_request(
		String::from("https://hiperdex.com"),
		request.header("User-Agent", USER_AGENT),
	);
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	template::handle_url(url, get_data())
}
