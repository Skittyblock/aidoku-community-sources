#![no_std]

use aidoku::{
	error::Result,
	helpers::substring::Substring,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, Manga, MangaPageResult, Page,
};

mod parser;
use parser::{get_filtered_url, request_get, BASE_URL, USER_AGENT};

extern crate alloc;
use alloc::string::ToString;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let url = get_filtered_url(filters, page);
	let html = request_get(url).html()?;

	parser::get_manga_list(html)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{}{}bz/", BASE_URL, id);
	let html = request_get(url).html()?;

	parser::get_manga_details(html, id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}{}bz/", BASE_URL, id);
	let html = request_get(url).html()?;

	parser::get_chapter_list(html)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!(
		"{}m{}/chapterimage.ashx?cid={1}&page=",
		BASE_URL, chapter_id
	);

	parser::get_page_list(url)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
		.header("Referer", BASE_URL)
		.header("User-Agent", USER_AGENT);
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let manga = url.contains("bz/").then(|| {
		let id = url
			.replace("bz/", "")
			.substring_after_last("/")
			.expect("manga id")
			.to_string();
		get_manga_details(id).expect("manga")
	});

	let chapter = url.contains(".com/m").then(|| {
		let id = url
			.substring_after_last("/m")
			.expect("chapter id")
			.replace('/', "");
		Chapter {
			id,
			..Default::default()
		}
	});

	Ok(DeepLink { manga, chapter })
}
