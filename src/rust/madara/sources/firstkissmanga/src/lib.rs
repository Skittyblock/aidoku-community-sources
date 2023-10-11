#![no_std]
use aidoku::{
	error::Result, prelude::*, std::net::Request, std::net::HttpMethod, std::String, std::Vec,
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
	helpers::substring::Substring
};

use madara_template::template;

extern crate base64;
use base64::{Engine as _, engine::general_purpose};

fn get_data() -> template::MadaraSiteData {
	let data: template::MadaraSiteData = template::MadaraSiteData {
		base_url: String::from("https://1st-kissmanga.net"),
		get_manga_id: get_int_manga_id,
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

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	template::handle_url(url, get_data())
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	template::modify_image_request(get_data().base_url, request)
}

fn get_int_manga_id(manga_id: String, base_url: String, path: String) -> String {
	let url = base_url + "/" + path.as_str() + "/" + manga_id.as_str();
	if let Ok(html) = Request::new(url.as_str(), HttpMethod::Get).html() {
		// For this web page, the script is encoded in base64
		let id_html_encoded = html.select("script#wp-manga-js-extra").attr("src").read();
		let id_html_base64 = id_html_encoded
			.substring_after("data:text/javascript;base64,").unwrap_or_default();
		let id_html_decoded = general_purpose::STANDARD.decode(id_html_base64).unwrap();
		let id_html = String::from_utf8(id_html_decoded).unwrap_or_default();

		let id = &id_html[id_html.find("manga_id").expect("Could not find manga_id") + 11
			..id_html.find("\"}").expect("Could not find end of manga_id")];
		String::from(id)
	} else {
		String::new()
	}
}
