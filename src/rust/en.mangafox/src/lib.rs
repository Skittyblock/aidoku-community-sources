#![no_std]
#![feature(test, let_chains, pattern)]
use aidoku::{
	error::Result, helpers::substring::Substring, prelude::*, std::net::HttpMethod,
	std::net::Request, std::String, std::Vec, Chapter, Filter, Manga, MangaPageResult, Page,
};

mod parser;
mod substring;
mod unpacker;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let url = parser::get_filtered_url(filters, page);
	println!("{}", &url);
	let html = Request::new(url.as_str(), HttpMethod::Get)
		.html()
		.expect("");
	parser::parse_directory(html)
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = String::from("https://www.fanfox.net/manga/") + &manga_id;
	let html = Request::new(url.as_str(), HttpMethod::Get)
		.html()
		.expect("");
	parser::parse_manga(html, manga_id)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = String::from("https://www.fanfox.net/manga/") + &manga_id;
	let html = Request::new(url.as_str(), HttpMethod::Get)
		.header("Cookie", "isAdult=1")
		.html()
		.expect("");
	parser::parse_chapters(html)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("https://m.fanfox.net/manga/{}", chapter_id);
	println!("->c {}", &url);
	let html = Request::new(url.as_str(), HttpMethod::Get)
		.header("Cookie", "readway=2")
		.html()
		.expect("");
	parser::get_page_list(html)
}

#[modify_image_request]
pub fn modify_image_request(request: Request) {
	println!("yuse");
	request.header("Referer", "https://m.fanfox.net/");
}

// #[handle_url]
// pub fn handle_url(url: String) -> Result<DeepLink> {
// 	let parsed_manga_id = parser::parse_incoming_url(url);

// 	Ok(DeepLink {
// 		manga: Some(get_manga_details(parsed_manga_id)?),
// 		chapter: None,
// 	})
// }
