#![no_std]

mod helper;
mod parser;
extern crate alloc;

use aidoku::{
	error::Result,
	prelude::{
		format, get_chapter_list, get_manga_details, get_manga_list, get_manga_listing,
		get_page_list,
	},
	std::{
		net::{HttpMethod, Request},
		*,
	},
	Chapter, Filter, FilterType, Listing, Manga, MangaPageResult, Page,
};
use helper::get_search_url;
const BASE_URL: &str = "https://mangamonks.com";
const TRAVERSE_PATH: &str = "manga";

use parser::{
	parse_chapter_list, parse_manga_details, parse_manga_list, parse_manga_listing, parse_page_list,
};

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let url = match listing.name.as_str() {
		"Latest" => format!("{}/latest-releases/{}", BASE_URL, page),
		"Popular" => format!("{}/popular-manga/{}", BASE_URL, page),
		"Admin's Choices" => format!("{}/admins-choices/{}", BASE_URL, page),
		"Most Viewed" => format!("{}/most-viewed/{}", BASE_URL, page),
		_ => format!("{}/new-arrivals/{}", BASE_URL, page),
	};
	let html = Request::new(url, HttpMethod::Get).html()?;
	parse_manga_listing(html)
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut included_tags: Vec<String> = Vec::new();
	let mut excluded_tags: Vec<String> = Vec::new();
	let mut status: String = String::new();
	let mut title: String = String::new();
	let mut manga_type: String = String::new();
	let status_options = ["", "ongoing", "completed"];
	let type_options = ["", "japanese", "korean", "chinese"];
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				title = filter.value.as_string()?.read();
			}
			FilterType::Genre => match filter.value.as_int().unwrap_or(-1) {
				0 => excluded_tags.push(filter.object.get("id").as_string()?.read()),
				1 => included_tags.push(filter.object.get("id").as_string()?.read()),
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

	if !title.is_empty() {
		let parameters = format!("dataType=json&phrase={}", title);
		let url = format!("{}/search/live", BASE_URL);
		let json = Request::new(url, HttpMethod::Post)
			.header("User-Agent", "Aidoku")
			.header(
				"Content-Type",
				"application/x-www-form-urlencoded; charset=UTF-8",
			)
			.header("Accept", "application/json")
			.header("X-Requested-With", "XMLHttpRequest")
			.body(parameters)
			.json()?
			.as_object()?;
		parse_manga_list(json)
	} else {
		let url = get_search_url(
			BASE_URL,
			included_tags,
			excluded_tags,
			manga_type,
			status,
			page,
		);
		let html = Request::new(url, HttpMethod::Get).html()?;
		parse_manga_listing(html)
	}
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let manga_url = format!("{BASE_URL}/{TRAVERSE_PATH}/{id}");
	let html = Request::new(manga_url, HttpMethod::Get).html()?;
	parse_manga_details(id, html)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("{BASE_URL}/{TRAVERSE_PATH}/{id}");
	let html = Request::new(url, HttpMethod::Get).html()?;
	parse_chapter_list(html)
}

#[get_page_list]
fn get_page_list(id: String, chapter: String) -> Result<Vec<Page>> {
	let url = format!("{BASE_URL}/{TRAVERSE_PATH}/{id}/{chapter}/all-pages");
	let html = Request::new(url, HttpMethod::Get).html()?;
	parse_page_list(html, BASE_URL)
}
