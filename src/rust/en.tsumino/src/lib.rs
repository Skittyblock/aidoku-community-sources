#![no_std]

use aidoku::{
	error::Result,
	helpers::uri::encode_uri,
	prelude::*,
	std::net::HttpMethod,
	std::net::Request,
	std::{String, Vec},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, Page,
};
extern crate alloc;
use alloc::{string::ToString, vec};
mod helper;
mod parser;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query: String = String::new();
	let mut sort: String = String::from("Newest");
	let mut tags: String = String::new();
	let mut tags_index = 0;
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query.push_str("&Text=");
				query.push_str(&encode_uri(filter.value.as_string()?.read()));
			}
			FilterType::Genre => {
				tags.push_str(&format!("&Tags[{tags_index}][Type]=1"));
				tags.push_str(&format!(
					"&Tags[{}][Text]={}",
					tags_index.to_string(),
					filter.name
				));
				match filter.value.as_int().unwrap_or(-1) {
					0 => tags.push_str(&format!("&Tags[{}][Exclude]=true", tags_index.to_string())),
					1 => {
						tags.push_str(&format!("&Tags[{}][Exclude]=false", tags_index.to_string()))
					}
					_ => continue,
				}
				tags_index += 1;
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int().unwrap_or(0) as i32;
				let option = match index {
					0 => "Newest",
					1 => "Oldest",
					2 => "Alphabetical",
					3 => "Rating",
					4 => "Pages",
					5 => "Views",
					6 => "Random",
					7 => "Comments",
					8 => "Popularity",
					_ => continue,
				};
				sort = String::from(option)
			}
			_ => continue,
		}
	}
	let url = String::from("https://www.tsumino.com/search/operate/");
	let mut parameters = String::new();
	parameters.push_str("PageNumber=");
	parameters.push_str(&encode_uri(page.to_string()));
	parameters.push_str(&query);
	parameters.push_str("&Sort=");
	parameters.push_str(&encode_uri(sort));
	parameters.push_str(&tags);
	let request = Request::new(&url, HttpMethod::Post)
		.header("User-Agent", "Aidoku")
		.body(parameters);
	let json = request.json()?.as_object()?;
	let data = json.get("data").as_array()?;

	let mut manga_arr: Vec<Manga> = Vec::new();
	for manga in data {
		let obj = match manga.as_object() {
			Ok(obj) => obj,
			Err(_) => continue,
		};
		if let Ok(manga) = parser::parse_list(obj) {
			manga_arr.push(manga);
		}
	}
	let total = json.get("pageCount").as_int().unwrap_or(0) as i32;

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: page < total,
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let url = String::from("https://www.tsumino.com/search/operate/");
	let mut parameters = String::new();
	parameters.push_str("PageNumber=");
	parameters.push_str(&encode_uri(page.to_string()));
	parameters.push_str("&Sort=");
	parameters.push_str(&encode_uri(listing.name));
	let request = Request::new(&url, HttpMethod::Post)
		.header("User-Agent", "Aidoku")
		.body(parameters);
	let json = request.json()?.as_object()?;
	let data = json.get("data").as_array()?;

	let mut manga_arr: Vec<Manga> = Vec::new();
	for manga in data {
		let obj = manga.as_object()?;
		if let Ok(manga) = parser::parse_list(obj) {
			manga_arr.push(manga);
		}
	}
	let total = json.get("pageCount").as_int().unwrap_or(0) as i32;

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: page < total,
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("https://www.tsumino.com/entry/{}", id);
	let request = Request::new(&url, HttpMethod::Get).header("User-Agent", "Aidoku");
	let html = request.html()?;
	let manga = parser::parse_manga(html)?;
	Ok(Manga {
		id,
		title: manga.title,
		cover: manga.cover,
		author: manga.author,
		description: manga.description,
		url,
		categories: manga.categories,
		status: manga.status,
		nsfw: manga.nsfw,
		viewer: manga.viewer,
		..Default::default()
	})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("https://www.tsumino.com/entry/{}", id);
	let request = Request::new(&url, HttpMethod::Get).header("User-Agent", "Aidoku");
	let html = request.html()?;
	let date_uploaded = html
		.select("div.book-info-container #Uploaded")
		.text()
		.0
		.as_date("yyyy MMMM d", Some("en_US"), None)
		.unwrap_or(-1.0);
	// skitty wants this to be put in
	let scanlator = html
		.select("div.book-info-container")
		.select("#Uploader")
		.select("a")
		.array()
		.get(0)
		.as_node()
		.expect("Failed to get uploader")
		.text()
		.read();

	Ok(vec![Chapter {
		id,
		title: String::from("Chapter 1"),
		volume: -1.0,
		chapter: 1.0,
		date_updated: date_uploaded,
		url,
		scanlator,
		lang: String::from("en"),
	}])
}

#[get_page_list]
fn get_page_list(id: String, _: String) -> Result<Vec<Page>> {
	let request = Request::new(
		format!("https://www.tsumino.com/Read/Index/{}", id),
		HttpMethod::Get,
	)
	.header("User-Agent", "Aidoku");
	let mut pages: Vec<Page> = Vec::new();
	let html = request.html()?;
	let num_pages = html
		.select("h1")
		.text()
		.read()
		.split(' ')
		.last()
		.unwrap()
		.parse::<i32>()
		.ok();
	match num_pages {
		Some(num_pages) => {
			for index in 1..(num_pages + 1) {
				let url = html
					.select("#image-container")
					.attr("data-cdn")
					.read()
					.replace("[PAGE]", &index.to_string());
				pages.push(Page {
					index,
					url,
					..Default::default()
				});
			}
			Ok(pages)
		}
		None => Err(aidoku::error::AidokuError {
			reason: aidoku::error::AidokuErrorKind::NodeError(aidoku::error::NodeError::ParseError),
		}),
	}
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let numbers: Vec<&str> = url.rsplitn(2, '/').collect();
	let id = numbers[0].parse::<i32>().ok().unwrap();
	let manga = get_manga_details(id.to_string());
	Ok(DeepLink {
		manga: Some(manga?),
		chapter: None,
	})
}
