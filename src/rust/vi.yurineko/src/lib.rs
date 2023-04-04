#![no_std]
mod helper;
mod parser;
use crate::{
	helper::{get_search_url, get_tag_id, listing_map, urlencode},
	parser::{parse_chapter, parse_manga},
};
use aidoku::{
	error::Result,
	prelude::*,
	std::{defaults::defaults_get, net::HttpMethod, net::Request, String, Vec},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, Page,
};

#[get_manga_list]
pub fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut genre = String::new();
	let mut query = String::new();
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = urlencode(filter.value.as_string()?.read());
			}
			_ => match filter.name.as_str() {
				"Tag" => {
					genre = get_tag_id(filter.value.as_int().unwrap_or(0));
				}
				_ => continue,
			},
		}
	}
	let search_url = get_search_url(String::from("https://api.yurineko.net"), query, genre, page);
	let json = Request::new(search_url.as_str(), HttpMethod::Get)
		.json()?
		.as_object()?;
	let result = json.get("result").as_array()?;
	let total = json.get("resultCount").as_int().unwrap_or(0);
	let mut manga_arr: Vec<Manga> = Vec::new();
	for manga in result {
		let manga_obj = manga.as_object()?;
		if let Ok(manga) = parse_manga(manga_obj) {
			manga_arr.push(manga);
		}
	}

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: i64::from((page - 1) + 20) < total,
	})
}

#[get_manga_listing]
pub fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let url = format!(
		"https://api.yurineko.net/{}?page={page}",
		listing_map(listing.name)
	);

	let result = Request::new(url.as_str(), HttpMethod::Get)
		.json()?
		.as_array()?;
	let mut manga_arr: Vec<Manga> = Vec::new();
	for manga in result {
		let manga_obj = manga.as_object()?;
		if let Ok(manga) = parse_manga(manga_obj) {
			manga_arr.push(manga);
		}
	}

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: url.contains("random"),
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("https://api.yurineko.net/manga/{id}");
	let json = Request::new(url.as_str(), HttpMethod::Get)
		.json()?
		.as_object()?;
	parse_manga(json)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("https://api.yurineko.net/manga/{id}");

	let json = Request::new(url.as_str(), HttpMethod::Get)
		.json()?
		.as_object()?;
	let chapters = json.get("chapters").as_array()?;
	let scanlators = json.get("team").as_array()?;
	let scanlators_string = scanlators
		.map(|a| {
			let scanlator_object = a.as_object()?;
			Ok(scanlator_object.get("name").as_string()?.read())
		})
		.map(|a: Result<String>| a.unwrap_or_default())
		.collect::<Vec<String>>()
		.join(", ");

	let mut chapter_arr: Vec<Chapter> = Vec::new();
	for chapter in chapters {
		let chapter_obj = chapter.as_object()?;
		if let Ok(chapter) = parse_chapter(String::from(&scanlators_string), chapter_obj) {
			chapter_arr.push(chapter);
		}
	}
	Ok(chapter_arr)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("https://api.yurineko.net/read/{chapter_id}");
	let mut request = Request::new(url.as_str(), HttpMethod::Get)
		.header(
			"Authorization", 
			"Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6MjE2NjMwLCJyb2xlIjoxLCJpYXQiOjE2NTI3MDk5MzYsImV4cCI6MTY1Nzg5MzkzNn0.q4NSW_AaWnlMJgSYkN9yE__wxpiD2aXDN82cdozfODg"
		);
	if let Ok(r18_token_val) = defaults_get("r18Token") {
		if let Ok(r18_token) = r18_token_val.as_string() {
			request = Request::new(url.as_str(), HttpMethod::Get).header(
				"Authorization",
				format!("Bearer {}", r18_token.read()).as_str(),
			);
		}
	}
	let json = request.json()?.as_object()?;
	let pages = json.get("url").as_array()?;
	let mut page_arr: Vec<Page> = Vec::new();
	for (idx, page) in pages.enumerate() {
		page_arr.push(Page {
			index: idx as i32,
			url: page.as_string()?.read(),
			base64: String::new(),
			text: String::new(),
		})
	}
	Ok(page_arr)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.header("Referer", "https://yurineko.net")
		.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.0.4951.54 Safari/537.36 Edg/101.0.1210.39");
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let url = &url[21..]; // remove https://yurineko.net/

	if url.starts_with("manga") {
		// https://yurineko.net/manga/419
		let id = &url[6..]; // remove manga/
		let manga = get_manga_details(String::from(id))?;
		return Ok(DeepLink {
			manga: Some(manga),
			chapter: None,
		});
	} else if url.starts_with("read") {
		// https://yurineko.net/read/419/5473
		let id = &url[5..]; // remove read/
		let end = match id.find('/') {
			Some(end) => end,
			None => id.len(),
		};
		let manga_id = &id[..end];
		let manga = get_manga_details(String::from(manga_id))?;

		let api_url = format!("https://api.yurineko.net/read/{id}");
		let json = Request::new(api_url.as_str(), HttpMethod::Get)
			.json()?
			.as_object()?;
		let chapter_info = json.get("chapterInfo").as_object()?;
		let chapter = parse_chapter(String::from(""), chapter_info)?;

		return Ok(DeepLink {
			manga: Some(manga),
			chapter: Some(chapter),
		});
	}
	Err(aidoku::error::AidokuError {
		reason: aidoku::error::AidokuErrorKind::Unimplemented,
	})
}
