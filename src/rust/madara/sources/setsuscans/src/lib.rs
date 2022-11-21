#![no_std]
use aidoku::{
	error::Result, prelude::*, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

use madara_template::template;

fn get_data() -> template::MadaraSiteData {
	let data: template::MadaraSiteData = template::MadaraSiteData {
		base_url: String::from("https://setsuscans.com"),
		description_selector: String::from("div.summary_content_wrap div p"),
		get_manga_id,
		alt_ajax: true,
		..Default::default()
	};
	data
}

// SetsuScans keeps its manga data in its src attribute as a base64 encoded string
// so this function overrides the default get_manga_id function to handle SetsuScans
fn get_manga_id(manga_id: String, base_url: String, path: String) -> String {
	let url = base_url + "/" + path.as_str() + "/" + manga_id.as_str();
	let html = Request::new(url.as_str(), HttpMethod::Get).html();
	let id_html = html.select("script#wp-manga-js-extra").attr("src").read();
	let id_html = id_html.replace("data:text/javascript;base64,", "");
	let decoded_html = base64::decode(id_html).expect("Failed to decode base64");
	let decoded_html = String::from_utf8(decoded_html).expect("Failed to convert base64 to utf8");
	let id = &decoded_html[decoded_html
		.find("manga_id")
		.expect("Failed to find manga_id")
		+ 11
		..decoded_html
			.find("\"}")
			.expect("Failed to find end of manga_id")];
	String::from(id)
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
fn get_page_list(id: String) -> Result<Vec<Page>> {
	template::get_page_list(id, get_data())
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	template::handle_url(url, get_data())
}
