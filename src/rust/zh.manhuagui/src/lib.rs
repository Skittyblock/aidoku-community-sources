#![no_std]
extern crate alloc;

mod decoder;
mod helper;
mod parser;

use aidoku::{
	error::Result, prelude::*,
	std::{defaults::defaults_get, net::HttpMethod, net::Request, String, Vec},
	Chapter, Filter, Manga, MangaPageResult, Page,
};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";

fn get_base_url() -> &'static str {
	let charset_index = defaults_get("charset")
		.and_then(|value| value.as_int())
		.unwrap_or(0);
	match charset_index {
		0 => "https://www.manhuagui.com",
		1 => "https://tw.manhuagui.com",
		_ => "https://www.manhuagui.com",
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut url = String::new();

	parser::get_filtered_url(filters, page, &mut url);

	let request = Request::new(url.as_str(), HttpMethod::Get)
		.header("Referer", get_base_url())
		.header("User-Agent", USER_AGENT)
		.header("Accept-Language", "zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7")
		.header("Cookie", "device_view=pc");
	let html = request.html()?;
	if url.contains(&format!("{}/list/", get_base_url())) {
		return parser::parse_home_page(html);
	}
	parser::parse_search_page(html)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{}/comic/{}", get_base_url(), id);
	let request = Request::new(url.as_str(), HttpMethod::Get)
		.header("Referer", get_base_url())
		.header("User-Agent", USER_AGENT)
		.header("Accept-Language", "zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7")
		.header("Cookie", "device_view=pc");
	let html = request.html()?;
	parser::parse_manga_details(html, id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/comic/{}", get_base_url(), id);
	let request = Request::new(url.as_str(), HttpMethod::Get)
		.header("Referer", get_base_url())
		.header("User-Agent", USER_AGENT)
		.header("Accept-Language", "zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7")
		.header("Cookie", "device_view=pc");
	let html = request.html()?;
	parser::get_chapter_list(html)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let base_url = format!("{}/comic/{}/{}.html", get_base_url(), manga_id, chapter_id);
	parser::get_page_list(base_url)
}

#[modify_image_request]
pub fn modify_image_request(request: Request) {
	let _ = request
		.header("Referer", get_base_url())
		.header("Accept-Language", "zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7")
		.header("Cookie", "device_view=pc");
}
