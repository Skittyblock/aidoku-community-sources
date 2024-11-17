#![no_std]

mod helper;
mod parser;

use aidoku::{
	error::Result,
	helpers::uri::encode_uri_component,
	prelude::*,
	std::{
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, Page,
};

use helper::*;
use parser::*;

const BASE_URL: &str = "https://ciorti.online";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut url = format!("{}/advanced.php?list={}", BASE_URL, page);

	let mut searching = false;

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(value) = filter.value.as_string() {
					searching = true;
					let query = encode_uri_component(value.read());
					url = format!("{}/search.php?manga={}", BASE_URL, query);
					break;
				}
			}
			FilterType::Genre => {
				if let Ok(id) = filter.object.get("id").as_string() {
					url.push_str(format!("&genre%5B%5D={}", id.read()).as_str());
				}
			}
			FilterType::Select => match filter.name.as_str() {
				"Status" => match filter.value.as_int().unwrap_or(-1) {
					0 => url.push_str("&status=all"),
					1 => url.push_str("&status=ongoing"),
					2 => url.push_str("&status=completed"),
					_ => continue,
				},
				_ => continue,
			},
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int().unwrap_or(0);
				let ascending = value.get("ascending").as_bool().unwrap_or(false);
				url.push_str(match (index, ascending) {
					(0, true) => "&orderby=NAME%20ASC",
					(0, false) => "&orderby=NAME%20DESC",
					(1, true) => "&orderby=VIEWS%20ASC",
					(1, false) => "&orderby=VIEWS%20DESC",
					_ => continue,
				});
			}
			_ => continue,
		}
	}

	let html = Request::new(url, HttpMethod::Get).html()?;

	Ok(parse_manga_list(html, searching))
}

#[get_manga_listing]
fn get_manga_listing(_listing: Listing, page: i32) -> Result<MangaPageResult> {
	let url = format!("{}/lastupdates.php?list={}", BASE_URL, page);

	let html = Request::new(url, HttpMethod::Get).html()?;

	Ok(parse_latest_manga_list(html))
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = get_manga_url(&manga_id);

	let html = Request::new(url.clone(), HttpMethod::Get).html()?;

	Ok(parse_manga_details(html, url))
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = get_manga_url(&manga_id);

	let html = Request::new(url, HttpMethod::Get).html()?;

	Ok(parse_chapter_list(html))
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let chap_url = get_chapter_url(&chapter_id)?;

	let html = Request::new(chap_url, HttpMethod::Get).html()?;

	Ok(parse_page_list(html))
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.header("Referer", BASE_URL);
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let manga_id = get_manga_id(&url);
	let chapter_id = get_chapter_id(&url);

	Ok(DeepLink {
		manga: get_manga_details(manga_id).ok(),
		chapter: Some(Chapter {
			id: chapter_id,
			..Default::default()
		}),
	})
}
