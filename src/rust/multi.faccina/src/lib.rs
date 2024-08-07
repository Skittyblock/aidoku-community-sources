#![no_std]
mod dto;
extern crate alloc;
use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	helpers::uri::{encode_uri, encode_uri_component},
	prelude::*,
	std::{defaults::defaults_get, net::Request, String, Vec},
	Chapter, Filter, FilterType, Manga, MangaPageResult, Page,
};
use alloc::{string::ToString, vec};
use dto::{ArchiveDto, ArchiveListDto, PageWrapperDto};

fn get_api_key() -> String {
	defaults_get("apiKey")
		.and_then(|v| v.as_string().map(|v| v.read()))
		.unwrap_or_default()
}

fn get_base_url() -> Result<String> {
	defaults_get("baseURL")?
		.as_string()
		.map(|v| v.read().trim_end_matches('/').to_string())
}

fn get_cdn_url() -> Result<String> {
	defaults_get("cdnURL")?
		.as_string()
		.map(|v| v.read().trim_end_matches('/').to_string())
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let base_url = get_base_url()?;
	let mut url = base_url.clone();
	url.push_str("/api/library?page=");
	url.push_str(itoa::Buffer::new().format(page));
	for filter in filters {
		match filter.kind {
			FilterType::Check => {
				if let Ok(id) = filter.object.get("id").as_string() {
					url.push_str(&id.read());
				}
			}
			FilterType::Sort => {
				if let Ok(value) = filter.value.as_object() {
					let index = value.get("index").as_int().unwrap_or(0);
					let ascending = value.get("ascending").as_bool().unwrap_or(true);
					let property = match index {
						0 => "released_at",
						1 => "created_at",
						2 => "title",
						3 => "pages",
						_ => continue,
					};
					url.push_str("&sort=");
					url.push_str(property);
					url.push_str("&order=");
					url.push_str(if ascending { "asc" } else { "desc" });
				}
			}
			FilterType::Title => {
				if let Ok(title) = filter.value.as_string() {
					let title = title.read();
					url.push_str("&q=");
					url.push_str(&encode_uri_component(title));
				}
			}
			_ => continue,
		}
	}

	let data = Request::get(encode_uri(url))
		.header("X-Api-Key", &get_api_key())
		.data();
	serde_json::from_slice(&data)
		.map(|v: PageWrapperDto<ArchiveListDto>| MangaPageResult {
			manga: v
				.archives
				.into_iter()
				.map(|v| v.into_manga(&base_url))
				.collect::<Vec<_>>(),
			has_more: v.total > (v.page * v.limit) as i64,
		})
		.map_err(|_| AidokuError {
			reason: AidokuErrorKind::JsonParseError,
		})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let base_url = get_base_url()?;
	let url = format!("{base_url}/api/g/{id}");
	let data = Request::get(encode_uri(url))
		.header("X-Api-Key", &get_api_key())
		.data();
	serde_json::from_slice(&data)
		.map(|v: ArchiveDto| v.into_manga(&base_url))
		.map_err(|_| AidokuError {
			reason: AidokuErrorKind::JsonParseError,
		})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let base_url = get_base_url()?;
	let url = format!("{base_url}/g/{id}");
	Ok(vec![Chapter {
		id,
		title: "Chapter".to_string(),
		chapter: 1.0,
		url,
		..Default::default()
	}])
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	if let Ok(base_url) = get_base_url() {
		request
			.header("Referer", &base_url)
			.header("Origin", &base_url);
	}
}

#[get_page_list]
fn get_page_list(_: String, id: String) -> Result<Vec<Page>> {
	let base_url = get_base_url()?;
	let cdn_url = get_cdn_url()?;
	let url = format!("{base_url}/api/g/{id}");
	let data = Request::get(encode_uri(url))
		.header("X-Api-Key", &get_api_key())
		.data();
	serde_json::from_slice(&data)
		.map(|v: ArchiveDto| v.into_pages(&cdn_url))
		.map_err(|_| AidokuError {
			reason: AidokuErrorKind::JsonParseError,
		})
}
