use aidoku::{
	error::Result,
	prelude::format,
	std::{String, Vec},
	Filter, FilterType, MangaPageResult,
};

use mangastream_template::helper::*;
use mangastream_template::template::MangaStreamSource;

pub fn get_listing_url(source: &MangaStreamSource, listing_name: String, page: i32) -> String {
	let code = get_lang_code();

	let list_type = if listing_name == source.listing[0] && code == "ko" {
		"genre%5B%5D=raw&type=manhwa&order=update"
	} else if listing_name == source.listing[1] && code == "ko" {
		"genre%5B%5D=raw&type=manhwa&order=popular"
	} else if listing_name == source.listing[2] && code == "ko" {
		"genre%5B%5D=raw&type=manhwa&order=latest"
	} else {
		&source.base_url
	};
	match page {
		1 if list_type != source.base_url => {
			format!(
				"{}/{}/?{}",
				source.base_url, source.traverse_pathname, list_type
			)
		}
		page if page > 1 => format!(
			"{}/{}/?page={}&{}",
			source.base_url, source.traverse_pathname, page, list_type
		),
		_ => source.base_url.clone(),
	}
}

pub fn get_title_skip() -> Vec<String> {
	match get_lang_code().as_str() {
		"ko" => ["chinese".into()].to_vec(),
		_ => ["chinese".into(), "raw".into()].to_vec(),
	}
}

pub fn parse_manga_list(
	source: &MangaStreamSource,
	filters: Vec<Filter>,
	page: i32,
) -> Result<MangaPageResult> {
	let mut included_tags: Vec<String> = Vec::new();
	let mut status: String = String::new();
	let mut title: String = String::new();
	let mut manga_type: String = String::new();
	let status_options = ["", "ongoing", "completed", "hiatus"];
	let type_options = ["", "manga", "manhwa", "manhua", "comic"];
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				title = filter.value.as_string()?.read();
			}
			FilterType::Genre => match filter.value.as_int().unwrap_or(-1) {
				1 => {
					if let Ok(id) = filter.object.get("id").as_string() {
						included_tags.push(id.read());
					}
				}
				_ => continue,
			},

			FilterType::Select => {
				let index = filter.value.as_int().unwrap_or(-1) as usize;
				match filter.name.as_str() {
					"Status" => status = String::from(status_options[index]),
					"Type" => manga_type = String::from(type_options[index]),
					_ => continue,
				}
			}
			_ => continue,
		};
	}
	let url = get_search_url(source, title, page, included_tags, status, manga_type);
	source.parse_manga_listing(url, String::from("Latest"), page)
}

pub fn get_search_url(
	source: &MangaStreamSource,
	query: String,
	page: i32,
	included_tags: Vec<String>,
	status: String,
	manga_type: String,
) -> String {
	let mut url = format!("{}/{}", source.base_url, source.traverse_pathname);
	if query.is_empty() && included_tags.is_empty() && status.is_empty() && manga_type.is_empty() {
		return get_listing_url(source, String::from(source.listing[0]), page);
	}
	if !query.is_empty() {
		url.push_str(&format!("/page/{}?s={}", page, query.replace(' ', "+")))
	} else {
		url.push_str(&format!("/?page={}", page));
	}
	if !included_tags.is_empty() {
		for tag in included_tags {
			url.push_str(&format!("&genre%5B%5D={}", tag));
		}
	}
	if !status.is_empty() {
		url.push_str(&format!("&status={}", status));
	}
	if !manga_type.is_empty() {
		url.push_str(&format!("&type={}", manga_type));
	}
	url
}
