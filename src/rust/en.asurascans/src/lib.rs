#![no_std]

mod helper;
mod parser;

use aidoku::{
	error::Result,
	helpers::uri::encode_uri_component,
	prelude::*,
	std::defaults::defaults_get,
	std::net::{HttpMethod, Request},
	std::{String, Vec},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaPageResult, Page,
};

use helper::*;
use parser::*;

const BASE_URL: &str = "https://asuracomic.net";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut url = format!("{}/series?page={}", BASE_URL, page);

	let mut genres = Vec::new();

	// '-1' means 'All', its's the default value for generes, status, and types
	// 'update' is the default value for order
	// All the filters are returned as JSON from this endpoint:
	// https://gg.asuracomic.net/api/series/filters
	// In the future source api rewrite, we can utilize this endpoint to dynamically
	// set the filters and their values.
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(value) = filter.value.as_string() {
					let query = encode_uri_component(value.read());
					url.push_str(format!("&name={query}").as_str());
				}
			}
			FilterType::Genre => {
				if let Ok(id) = filter.object.get("id").as_string() {
					match filter.value.as_int().unwrap_or(-1) {
						1 => genres.push(id.read()),
						_ => continue,
					}
				}
			}
			FilterType::Select => match filter.name.as_str() {
				"Status" => match filter.value.as_int().unwrap_or(-1) {
					1 => url.push_str("&status=1"),
					2 => url.push_str("&status=2"),
					3 => url.push_str("&status=3"),
					4 => url.push_str("&status=4"),
					5 => url.push_str("&status=5"),
					6 => url.push_str("&status=6"),
					_ => url.push_str("&status=-1"),
				},
				"Type" => match filter.value.as_int().unwrap_or(-1) {
					1 => url.push_str("&types=1"),
					2 => url.push_str("&types=2"),
					3 => url.push_str("&types=3"),
					_ => url.push_str("&types=-1"),
				},
				_ => continue,
			},
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int().unwrap_or(-1);
				match index {
					0 => url.push_str("&order=rating"),
					1 => url.push_str("&order=update"),
					2 => url.push_str("&order=latest"),
					3 => url.push_str("&order=desc"),
					4 => url.push_str("&order=asc"),
					_ => url.push_str("&order=update"),
				}
			}
			_ => continue,
		}
	}

	if !genres.is_empty() {
		url.push_str("&genres=");
		url.push_str(&genres.join(","));
	} else {
		url.push_str("&genres=-1");
	}

	let html = Request::new(url, HttpMethod::Get).html();

	Ok(parse_manga_list(html?))
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let manga_url = get_manga_url(&manga_id);

	let html = Request::new(manga_url, HttpMethod::Get).html()?;

	Ok(parse_manga_details(html, manga_id))
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
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
