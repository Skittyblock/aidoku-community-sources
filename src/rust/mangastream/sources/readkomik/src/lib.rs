#![no_std]
use aidoku::{
	error::Result, std::json::parse, prelude::*, std::net::{ Request,HttpMethod}, std::String, std::Vec, Chapter, DeepLink, Filter,
	Listing, Manga, MangaPageResult, Page,
};

use mangastream_template::{template::MangaStreamSource, helper::urlencode};

fn get_instance() -> MangaStreamSource {
	MangaStreamSource {
		base_url: String::from("https://readkomik.com"),
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

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	let html = Request::new(&id, HttpMethod::Get)
		.header("Referer", &get_instance().base_url)
		.html();
	let raw_text = html.select("script").html().read();
	let trimmed_text = &raw_text[raw_text.find(r#":[{"s"#).unwrap_or(0) + 2
		..raw_text.rfind("}],").unwrap_or(0) + 1];
	let json = parse(trimmed_text.as_bytes()).as_object()?;
	let images = json.get("images").as_array()?;
	for (index, page) in images.enumerate() {
		let page_url = urlencode(page.as_string()?.read());
		pages.push(Page {
			index: index as i32,
			url: page_url,
			base64: String::new(),
			text: String::new(),
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
