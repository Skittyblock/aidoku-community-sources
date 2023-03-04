#![no_std]

mod helper;
extern crate alloc;

use aidoku::{
	error::Result,
	prelude::{format, get_chapter_list, get_manga_details, get_manga_list, get_page_list},
	std::{
		net::{HttpMethod, Request},
		*,
	},
	Chapter, Filter, FilterType, Manga, MangaPageResult, Page,
};
use alloc::string::ToString;
use helper::{genre_id_from_filter, create_advanced_search_body, change_page, parse_search, BASE_URL, parse_chapter_list, parse_manga, parse_page_list};



#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut manga_arr: Vec<Manga> = Vec::new();

	let mut manga_title = String::new();
	let mut artist_name = String::new();
	let mut status: i64 = 0;
	let mut tag_search_mode = String::from("and");

	let mut included_tags: Vec<i64> = Vec::new();
	let mut excluded_tags: Vec<i64> = Vec::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => manga_title = filter.value.as_string()?.read(),
			FilterType::Author => artist_name = filter.value.as_string()?.read(),
			FilterType::Genre => {
				let object_id = filter.object.get("id").as_string()?.read();
				let object_type = object_id.split('_').next().unwrap_or("unknown");
				let object_value = genre_id_from_filter(&object_id);

				match object_type {
					"category" => match filter.value.as_int().unwrap_or(-1) {
						0 => excluded_tags.push(object_value),
						1 => included_tags.push(object_value),
						_ => continue,
					},
					"tag" => match filter.value.as_int().unwrap_or(-1) {
						0 => excluded_tags.push(object_value),
						1 => included_tags.push(object_value),
						_ => continue,
					},
					"doujin" => match filter.value.as_int().unwrap_or(-1) {
						0 => excluded_tags.push(object_value),
						1 => included_tags.push(object_value),
						_ => continue,
					},
					_ => continue,
				}
			}
			FilterType::Select => match filter.name.as_str() {
				"Status" => status = filter.value.as_int().unwrap(),
				"Tag Search Mode" => {
					tag_search_mode = match filter.value.as_int().unwrap_or(-1) {
						0 => String::from("and"),
						1 => String::from("or"),
						_ => String::from("and"),
					}
				}
				_ => continue,
			},
			_ => continue,
		}
	}

	let url = format!("{BASE_URL}/hentai-list/advanced-search/");

	let body_data = create_advanced_search_body(
		Some(manga_title),
		Some(artist_name),
		Some(status),
		Some(tag_search_mode),
		Some(included_tags),
	);

	let mut has_next = false;

	if let Ok(html) = Request::new(url, HttpMethod::Post).body(body_data).html() {
		let paging = html.select(".pagination");

		let mut next_page_url = String::new();

		if !paging.html().read().is_empty() {
			let next_page_node = paging.select("a#js-linkNext");
			if !next_page_node.html().read().is_empty() {
				next_page_url = next_page_node.attr("href").to_string();
			}

			if !next_page_url.is_empty() {
				has_next = true;
			}
		}

		if page > 1 {
			let next_page = change_page(&next_page_url, page);
			if let Ok(html) = Request::new(next_page, HttpMethod::Get).html() {
				manga_arr = parse_search(&html);
			}
		} else {
			manga_arr = parse_search(&html);
		}
	}

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: has_next,
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let manga_url = format!("{BASE_URL}/{id}");

	let html = Request::new(manga_url, HttpMethod::Get).html()?;
 	parse_manga(id, html)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("{BASE_URL}/{id}");

	match Request::new(url, HttpMethod::Get).html() {
		Ok(html) => parse_chapter_list(&html),
		Err(_) => Ok(Vec::new()),
	}
}

#[get_page_list]
fn get_page_list(id: String, chapter: String) -> Result<Vec<Page>> {
	let url = format!("{BASE_URL}/{id}/{chapter}/1");

	match Request::new(url, HttpMethod::Get).html() {
		Ok(html) => parse_page_list(&html),
		Err(_) => Ok(Vec::new()),
	}
}
