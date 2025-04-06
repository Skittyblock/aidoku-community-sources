use crate::constants;
use aidoku::helpers::uri::QueryParameters;
use aidoku::std::ValueRef;
use aidoku::{
	error::Result, helpers::uri::encode_uri, prelude::*, std::net::Request, Filter, FilterType,
	Manga, MangaPageResult,
};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub fn get_manga_url(id: &str) -> String {
	format!("{}/{}", constants::BASE_API_URL, encode_uri(id))
}

pub fn get_chapter_url(manga_id: &str, chapter_id: &str) -> String {
	format!(
		"{}/chapter/{}",
		get_manga_url(manga_id),
		encode_uri(chapter_id)
	)
}

pub fn fetch_json<T: AsRef<str>>(url: T) -> Result<ValueRef> {
	let request = Request::get(url)
		.header("User-Agent", constants::USER_AGENT)
		.header("Referer", constants::BASE_URL);

	Ok(request.json()?.as_object()?.get("response"))
}

pub fn get_search_url(filters: Vec<Filter>, page: i32) -> String {
	let mut query = QueryParameters::new();
	let mut genres: Vec<String> = Vec::new();
	let mut kinds: Vec<String> = Vec::new();
	let mut statuses: Vec<String> = Vec::new();
	let mut order = String::new();

	if page > 1 {
		query.push("page", Some(page.to_string().as_str()));
	}
	query.push("limit", Some(constants::PAGE_LIMIT.to_string().as_str()));

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					query.push("search", Some(filter_value.read().as_str()));
				}
			}
			FilterType::Genre => {
				if let Ok(obj_id) = filter.object.get("id").as_string() {
					let id = obj_id.read();
					match filter.value.as_int().unwrap_or(-1) {
						0 => genres.push(format!("!{}", id)),
						1 => genres.push(id),
						_ => continue,
					}
				}
			}
			FilterType::Check => {
				if let Ok(obj_id) = filter.object.get("id").as_string() {
					let id = obj_id.read();
					if let Some((kind, value)) = id.split_once('|') {
						match kind {
							"0" => kinds.push(value.to_string()),
							"1" => statuses.push(value.to_string()),
							_ => continue,
						}
					}
				}
			}
			FilterType::Sort => {
				if let Ok(value) = filter.value.as_object() {
					let index = value.get("index").as_int().unwrap_or(0);
					order.push_str(match index {
						0 => "id",
						1 => "name",
						2 => "popular",
						_ => "updated", // по обновлению (idx: 3), default
					});
				}
			}
			_ => continue,
		}
	}

	if !genres.is_empty() {
		query.push("genres", Some(genres.join(",").as_str()));
	}

	if !kinds.is_empty() {
		query.push("kinds", Some(kinds.join(",").as_str()));
	}

	if !statuses.is_empty() {
		query.push("status", Some(statuses.join(",").as_str()));
	}

	if !order.is_empty() {
		query.push("order", Some(order.as_str()));
	}

	format!("{}?{}", constants::BASE_API_URL, query.to_string()).to_string()
}

pub fn create_manga_page_result(mangas: Vec<Manga>) -> MangaPageResult {
	let has_more = !mangas.is_empty();
	MangaPageResult {
		manga: mangas,
		has_more,
	}
}
