#![no_std]
use aidoku::{
	error::Result, prelude::*, std::net::Request, std::String, std::Vec, Chapter, DeepLink, Filter,
	Listing, Manga, MangaPageResult, Page,
};
use base64::prelude::*;
use mangastream_template::template::{MangaStreamSource, USER_AGENT};

const BASE_URL: &str = "https://fl-ares.com";

fn get_instance() -> MangaStreamSource {
	MangaStreamSource {
		base_url: BASE_URL.into(),
		listing: ["الرائج", "آخر", "جَديد"],
		last_page_text: "التالي",
		traverse_pathname: "series",
		manga_details_author: ".imptdt:contains(المؤلف) i",
		manga_details_status: ".imptdt:contains(Status) i, .imptdt:contains(الحالة) i",
		chapter_date_format: "MMMM d, yyyy",
		locale: "ar_EH",
		..Default::default()
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	get_instance().parse_manga_list(filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	get_instance().parse_manga_listing(get_instance().base_url, listing.name, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	get_instance().parse_manga_details(id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	get_instance().parse_chapter_list(id)
}

// parse page json data in base64
#[get_page_list]
fn get_page_list(_manga_id: String, id: String) -> Result<Vec<Page>> {
	let url = format!("{BASE_URL}/{id}");

	let mut pages: Vec<Page> = Vec::new();
	let html = Request::get(url)
		.header("Referer", BASE_URL)
		.header("User-Agent", USER_AGENT)
		.html()?;

	let raw_data = html
		.select("script[src^=data:text/javascript;base64,dHNfcmVhZGVyLnJ1bih7]")
		.first()
		.attr("src")
		.read();

	let start_pattern = "base64,";

	let Some(start_idx) = raw_data.find(start_pattern) else {
		return get_instance().parse_page_list(id);
	};

	let base64_data = &raw_data[start_idx + start_pattern.len()..];

	let Some(json) = BASE64_STANDARD
		.decode(base64_data)
		.ok()
		.and_then(|s| String::from_utf8(s).ok())
	else {
		return Ok(Vec::new());
	};

	let trimmed_json =
		&json[json.find(r#":[{"s"#).unwrap_or(0) + 2..json.rfind("}],").unwrap_or(2) + 1];

	let parsed_json = aidoku::std::json::parse(trimmed_json.as_bytes())?.as_object()?;
	let images = parsed_json.get("images").as_array()?;

	for (index, page) in images.enumerate() {
		let url = mangastream_template::helper::urlencode(page.as_string()?.read());
		pages.push(Page {
			index: index as i32,
			url,
			..Default::default()
		});
	}

	Ok(pages)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	get_instance().modify_image_request(request)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(url)
}
