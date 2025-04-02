use crate::constants;
use aidoku::std::{ValueRef};
use aidoku::{
	error::{Result},
	helpers::uri::encode_uri,
	prelude::*,
	std::net::{HttpMethod, Request},
	Filter, FilterType, Manga, MangaPageResult};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::parser::debug;

pub fn get_manga_url(id: &str) -> String {
	format!("{}/{}", constants::BASE_API_URL, encode_uri(id))
}

pub fn get_chapter_url(manga_id: &str, chapter_id: &str) -> String {
	format!("{}/chapter/{}", get_manga_url(manga_id), encode_uri(chapter_id))
}

pub fn fetch_json<T: AsRef<str>>(url: T, fetch_for: &str) -> Result<ValueRef>
{
	debug!("Fetching for {}, fetching url {}", fetch_for, url.as_ref());

	let request = Request::new(url, HttpMethod::Get)
		.header("User-Agent", constants::USER_AGENT)
		.header("Referer", constants::BASE_URL);

	Ok(request.json()
		.expect(format!("Failed to find JSON for {}", fetch_for).as_str())
		.as_object()?
		.get("response"))
}

pub fn get_search_url(filters: Vec<Filter>, page: i32) -> String
{
	debug!("Generating search");

	let mut url = String::from(constants::BASE_API_URL);
	let mut query: Vec<String> = Vec::new();
	let mut genres: Vec<String> = Vec::new();
	let mut kinds: Vec<String> = Vec::new();
	let mut statuses: Vec<String> = Vec::new();
	let mut order = String::new();

	if page > 1 {
		query.push(format!("page={}", page.to_string()));
	}
	query.push(format!("limit={}", constants::PAGE_LIMIT));

	for filter in filters {
		debug!("Filter: {}", filter.name);
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					query.push(format!("search={}", encode_uri(filter_value.read())));
				}
			},
			FilterType::Genre => {
				if let Ok(obj_id) = filter.object.get("id").as_string() {
					let id = obj_id.read();
					match filter.value.as_int().unwrap_or(-1) {
						0 => genres.push(format!("!{}", id)),
						1 => genres.push(id),
						_ => continue
					}
				}
			},
			FilterType::Check => {
				if let Ok(obj_id) = filter.object.get("id").as_string() {
					let id = obj_id.read();
					let parts: Vec<&str> = id.split('|').collect();
					if parts.len() < 2 {
						continue; // we must skip this since we don't know kind
					}

					let kind = parts[0];
					let value = parts.iter() // rebuild left part of value
						.skip(1)
						.map(|s| s.to_string())
						.collect::<Vec<String>>()
						.join("|");

					match kind {
						"0" => kinds.push(value),
						"1" => statuses.push(value),
						_ => continue
					}
				}
			},
			FilterType::Sort => {
				if let Ok(value) = filter.value.as_object() {
					let index = value.get("index").as_int().unwrap_or(0);
					order.push_str(match index {
						0 => "id",
						1 => "name",
						2 => "popular",
						_ => "updated" // по обновлению (idx: 3), default
					});
				}
			},
			_ => continue
		}
	}

	if !genres.is_empty() {
		query.push(format!("genres={}", encode_uri(genres.join(","))));
	}

	if !kinds.is_empty() {
		query.push(format!("kinds={}", encode_uri(kinds.join(","))));
	}

	if !statuses.is_empty() {
		query.push(format!("status={}", encode_uri(statuses.join(","))));
	}

	if !order.is_empty() {
		query.push(format!("order={}", encode_uri(order)));
	}

	url.push('?');
	url += &query.join("&");

	url
}

pub fn create_manga_page_result(mangas: Vec<Manga>) -> MangaPageResult {
	let has_more = !mangas.is_empty();
	MangaPageResult {
		manga: mangas,
		has_more,
	}
}
