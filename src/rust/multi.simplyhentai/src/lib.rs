#![no_std]

mod helper;
mod parser;
extern crate alloc;

use aidoku::{
	error::Result,
	prelude::{format, get_chapter_list, get_manga_details, get_manga_list, get_page_list},
	std::{
		defaults::defaults_get,
		net::{HttpMethod, Request},
	},
	Chapter, Filter, FilterType, Manga, MangaPageResult, Page,
};
use alloc::{string::String, vec::Vec};
use helper::{make_search_url, API_BASE_URL, USER_AGENT};
use parser::{parse_chapter_list, parse_manga, parse_page_list, parse_search};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut manga_title: Option<String> = None;
	let mut tags: Vec<String> = Vec::new();
	let mut sort: Option<&str> = None;
	let mut language = "English";
	if let Ok(languagesref) = defaults_get("languages") {
		if let Ok(mut languages) = languagesref.as_array() {
			if !languages.is_empty() {
				if let Some(lang) = languages.next() {
					language = match lang.as_string()?.read().as_str() {
						"en" => "English",
						"ja" => "Japanese",
						"fr" => "French",
						"de" => "German",
						"it" => "Italian",
						"ru" => "Russian",
						"es" => "Spanish",
						"ko" => "Korean",
						"pl" => "Polish",
						"zh" => "Chinese",
						_ => "English",
					}
				}
			}
		}
	}

	for filter in filters {
		match filter.kind {
			FilterType::Title => match filter.value.as_string() {
				Ok(title) => manga_title = Some(title.read()),
				_ => continue,
			},
			FilterType::Genre => {
				let genre = filter.object.get("name").as_string()?.read();
				tags.push(genre);
			}
			FilterType::Sort => {
				let sortobj = filter.value.as_object()?;
				let index = sortobj.get("index").as_int().unwrap_or(1);
				sort = match index {
					1 => Some("upload-date"),
					2 => Some("popularity"),
					_ => continue,
				}
			}
			_ => continue,
		}
	}

	let url = make_search_url(language, page, manga_title, tags, sort);

	let data = Request::new(url, HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.json()?;
	let res = data.as_object()?;

	parse_search(page, res)
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let manga_url = format!("{API_BASE_URL}/manga/{manga_id}");
	let data = Request::new(manga_url, HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.json()?;
	let res = data.as_object()?;
	parse_manga(manga_id, res)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let manga_url = format!("{API_BASE_URL}/manga/{manga_id}");
	let data = Request::new(manga_url, HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.json()?;
	let res = data.as_object()?;
	parse_chapter_list(manga_id, res)
}

#[get_page_list]
fn get_page_list(manga_id: String, _chapter_id: String) -> Result<Vec<Page>> {
	let manga_url = format!("{API_BASE_URL}/manga/{manga_id}/pages");
	let data = Request::new(manga_url, HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.json()?;
	let res = data.as_object()?;
	parse_page_list(res)
}
