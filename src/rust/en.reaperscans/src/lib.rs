#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::HttpMethod, net::Request, print, String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer,
};

mod helper;
mod parser;

const BASE_URL: &str = "https://reaperscans.com";

// TODO: Add search support, reaper uses a rest api for searching that uses a weird url format that could change at any time
// need to figure out a good way to deal with that, or steal tachiyomi's implementation
// https://reaperscans.com/livewire/message/frontend.dtddzhx-ghvjlgrpt
#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	parser::parse_manga_list(String::from(BASE_URL), filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	parser::parse_manga_listing(String::from(BASE_URL), listing, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	parser::parse_manga_details(String::from(BASE_URL), id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	let html = Request::new(id.as_str(), HttpMethod::Get)
		.html()
		.expect("i brokey");
	for chapter in html.select("main div[wire:id] div > ul > li").array() {
		let chapter_node = chapter.as_node().expect("now i brokey");
		let mut title = String::new();
		let parsed_title = chapter_node
			.select("div.min-w-0 div.text-sm p.font-medium")
			.text()
			.read();

		let mut chapter_number = -1.0;

		if parsed_title.contains('-') {
			title = String::from(parsed_title.split('-').collect::<Vec<&str>>()[1].trim());
			chapter_number = parsed_title
				.replace("Chapter", "")
				.split('-')
				.collect::<Vec<&str>>()[0]
				.trim()
				.parse::<f32>()
				.expect("i brokey");
		} else {
			chapter_number = parsed_title
				.replace("Chapter", "")
				.trim()
				.parse::<f32>()
				.expect("i brokey");
		}

		let chapter_id = chapter_node.select("a").attr("href").read();
		let chapter_url = chapter_node.select("a").attr("href").read();

		// let date = get_date(chapter_node.select(".episode-date").text().read());
		chapters.push(Chapter {
			id: chapter_id,
			title,
			volume: -1.0,
			chapter: chapter_number,
			date_updated: -1.0,
			scanlator: String::new(),
			url: chapter_url,
			lang: String::from("en"),
		});
	}
	Ok(chapters)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	todo!()
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	todo!()
}
