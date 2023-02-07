#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::net::{HttpMethod, Request},
	std::Vec,
	std::{json::parse, String},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

use mangastream_template::{helper::urlencode, template::MangaStreamSource};

fn get_instance() -> MangaStreamSource {
	MangaStreamSource {
		base_url: String::from("https://mangkomik.com/"),
		manga_title_trim: ["Bahasa Indonesia".into()].to_vec(),
		chapter_date_format: "MMMM d, yyyy",
		locale: "id",
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
fn get_page_list(_manga_id: String, id: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	let html = Request::new(&id, HttpMethod::Get)
		.header("Referer", &get_instance().base_url)
		.html()?;
	let externaljs = html.select("script[data-minify]").attr("src").read();
	let raw_text = Request::new(&externaljs, HttpMethod::Get).string()?;
	let trimmed_json = &raw_text
		[raw_text.find(r#":[{"s"#).unwrap_or(0) + 2..raw_text.rfind("}],").unwrap_or(0) + 1];
	let trimmed_text = if trimmed_json.contains("Default 2") {
		&trimmed_json[..trimmed_json.rfind(r#",{"s"#).unwrap_or(0)]
	} else {
		trimmed_json
	};
	let json = parse(trimmed_text.as_bytes())?.as_object()?;
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
