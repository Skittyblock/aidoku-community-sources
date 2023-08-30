#![no_std]
extern crate alloc;
mod parser;
mod url;

use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, MangaStatus, Page,
};
use alloc::string::ToString;
use parser::{MangaListResponse, NodeArrValue};
use url::Url;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let manga_list_url = Url::from((filters, page));
	let manga_list_request = Request::get(manga_list_url.to_string());

	if let Url::Filters { .. } = manga_list_url {
		let filters_page = manga_list_request.html()?;
		return filters_page.get_page_result();
	}

	let search_json = manga_list_request.json()?;
	search_json.get_page_result()
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	todo!()
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let manga_page = Request::get(Url::Manga(&manga_id).to_string()).html()?;

	let cover = manga_page
		.select("img.lazyload")
		.attr("data-src")
		.read()
		.replace(".328x422.jpg", "");

	let title = manga_page.select("h6").text().read();

	let artist = manga_page
		.select("span.comicParticulars-right-txt > a")
		.array()
		.filter_map(NodeArrValue::ok_text)
		.collect::<Vec<_>>()
		.join("、");

	let description = manga_page.select("p.intro").text().read();

	let manga_url = Url::Manga(&manga_id).to_string();

	let categories = manga_page
		.select("span.comicParticulars-left-theme-all.comicParticulars-tag > a")
		.array()
		.filter_map(NodeArrValue::ok_text)
		.map(|str| str[1..].to_string())
		.collect::<Vec<_>>();

	let status_str = manga_page
		.select("li:contains(狀態：) > span.comicParticulars-right-txt")
		.text()
		.read();
	let status = match status_str.as_str() {
		"連載中" => MangaStatus::Ongoing,
		"已完結" | "短篇" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};

	Ok(Manga {
		id: manga_id,
		cover,
		title,
		author: artist.clone(),
		artist,
		description,
		url: manga_url,
		categories,
		status,
		..Default::default()
	})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	todo!()
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	todo!()
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	todo!()
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	todo!()
}
