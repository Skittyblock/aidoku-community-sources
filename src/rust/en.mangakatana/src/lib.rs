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

const URL: &str = "https://mangakatana.com";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	// filter=1 enables filtering
	let mut url = format!("{}/manga/page/{}?filter=1", URL, page);

	let mut searching = false;
	let mut include_generes = Vec::new();
	let mut exclude_generes = Vec::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(value) = filter.value.as_string() {
					searching = true;
					let query = encode_uri_component(value.read());
					url = format!("{}/page/{}?search={}&search_by=book_name", URL, page, query);
				}
			}
			FilterType::Author => {
				if let Ok(value) = filter.value.as_string() {
					searching = true;
					let query = encode_uri_component(value.read());
					url = format!("{}/page/{}?search={}&search_by=author", URL, page, query);
				}
			}
			FilterType::Genre => {
				if let Ok(id) = filter.object.get("id").as_string() {
					match filter.value.as_int().unwrap_or(-1) {
						0 => exclude_generes.push(id.read()),
						1 => include_generes.push(id.read()),
						_ => continue,
					}
				}
			}
			FilterType::Select => match filter.name.as_str() {
				"Status" => match filter.value.as_int().unwrap_or(-1) {
					0 => continue,
					1 => url.push_str("&status=1"),
					2 => url.push_str("&status=2"),
					3 => url.push_str("&status=0"),
					_ => continue,
				},
				"Genre Inclusion Mode" => match filter.value.as_int().unwrap_or(-1) {
					0 => url.push_str("&include_mode=and"),
					1 => url.push_str("&include_mode=or"),
					_ => url.push_str("&include_mode=and"),
				},
				"Chapter Count" => match filter.value.as_int().unwrap_or(-1) {
					0 => url.push_str("&chapters=e1"),
					1 => url.push_str("&chapters=1"),
					2 => url.push_str("&chapters=5"),
					3 => url.push_str("&chapters=10"),
					4 => url.push_str("&chapters=20"),
					5 => url.push_str("&chapters=30"),
					6 => url.push_str("&chapters=50"),
					7 => url.push_str("&chapters=100"),
					8 => url.push_str("&chapters=150"),
					9 => url.push_str("&chapters=200"),
					_ => url.push_str("&chapters=1"),
				},
				_ => continue,
			},
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int().unwrap_or(0);
				url.push_str(match index {
					0 => "&order=az",
					1 => "&order=latest",
					2 => "&order=new",
					3 => "&order=numc",
					_ => "&order=latest",
				});
			}
			_ => continue,
		}
	}

	if !include_generes.is_empty() && !searching {
		url.push_str("&include=");
		url.push_str(&include_generes.join("_"));
	}

	if !exclude_generes.is_empty() && !searching {
		url.push_str("&exclude=");
		url.push_str(&exclude_generes.join("_"));
	}

	let html = Request::new(url, HttpMethod::Get)
		.html()
		.expect("Failed to get html from mangakatana");

	Ok(parse_manga_list(html, String::from(URL)))
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = get_manga_url(manga_id, String::from(URL));

	let html = Request::new(url.clone(), HttpMethod::Get)
		.html()
		.expect("Failed to get html from mangakatana");

	Ok(parse_manga_details(html, url))
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = get_manga_url(manga_id, String::from(URL));

	let html = Request::new(url, HttpMethod::Get)
		.html()
		.expect("Failed to get html from mangakatana");

	Ok(parse_chapter_list(html, String::from(URL)))
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let image_server = defaults_get("imageServer")?
		.as_string()
		.map(|v| v.read())
		.unwrap_or_default();

	let url = {
		let url = get_chapter_url(chapter_id, manga_id, String::from(URL));

		if image_server.is_empty() {
			url
		} else {
			format!("{}?sv={}", url, image_server)
		}
	};

	let html = Request::new(url, HttpMethod::Get)
		.html()
		.expect("Failed to get html from mangakatana");

	Ok(parse_page_list(html))
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.header("Referer", URL);
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let manga_id = get_manga_id(url.clone());
	let chapter_id = get_chapter_id(url);

	Ok(DeepLink {
		manga: get_manga_details(manga_id).ok(),
		chapter: Some(Chapter {
			id: chapter_id,
			..Default::default()
		}),
	})
}
