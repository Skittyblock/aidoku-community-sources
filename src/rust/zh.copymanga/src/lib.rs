#![no_std]
extern crate alloc;
mod decryptor;
mod parser;
mod url;

use aidoku::{
	error::Result,
	prelude::*,
	std::{json, net::Request, String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, MangaStatus, Page,
};
use alloc::string::ToString;
use decryptor::EncryptedString;
use parser::{MangaListResponse, NodeArrValue, UuidString};
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
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let mut chapters = Vec::<Chapter>::new();

	let mut chapter_vec = Vec::<(String, String, f64)>::new();
	let decrypted_results = Request::get(Url::ChapterList(&manga_id).to_string())
		.json()?
		.as_object()?
		.get("results")
		.as_string()?
		.read()
		.decrypt();
	let groups_values = json::parse(decrypted_results)?
		.as_object()?
		.get("groups")
		.as_object()?
		.values();
	for groups_value in groups_values {
		let chapters_arr = groups_value.as_object()?.get("chapters").as_array()?;
		for chapters_value in chapters_arr {
			let chapters_obj = chapters_value.as_object()?;

			let id = chapters_obj.get("id").as_string()?.read();
			let name = chapters_obj.get("name").as_string()?.read();
			let timestamp = id.get_timestamp();

			chapter_vec.push((id, name, timestamp));
		}
	}
	chapter_vec.sort_by(|a, b| a.2.total_cmp(&b.2));

	for (index, (chapter_id, title, date_updated)) in chapter_vec.iter().enumerate() {
		let chapter_num = (index + 1) as f32;
		let chapter_url = Url::Chapter(&manga_id, chapter_id).to_string();

		let chapter = Chapter {
			id: chapter_id.clone(),
			title: title.clone(),
			chapter: chapter_num,
			date_updated: *date_updated,
			url: chapter_url,
			lang: "zh".to_string(),
			..Default::default()
		};
		chapters.insert(0, chapter);
	}

	Ok(chapters)
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
