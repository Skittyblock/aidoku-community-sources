#![no_std]

use aidoku::{
	error::Result,
	prelude::*,
	std::net::HttpMethod,
	std::net::Request,
	std::{String, Vec},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, Page,
};
extern crate alloc;
use alloc::{string::ToString, vec};
mod helper;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query: String = String::new();
	let mut sort: String = String::from("Newest");
	let mut tags: String = String::new();
	let mut i = 0;
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query.push_str("&Text=");
				query.push_str(&helper::urlencode(filter.value.as_string()?.read()));
			}
			FilterType::Genre => {
				let tpe = 1;
				tags.push_str(&format!(
					"&Tags[{}][Type]={}",
					i.to_string(),
					tpe.to_string()
				));
				tags.push_str(&format!("&Tags[{}][Text]={}", i.to_string(), filter.name));
				match filter.value.as_int().unwrap_or(-1) {
					0 => tags.push_str(&format!("&Tags[{}][Exclude]=true", i.to_string())),
					1 => tags.push_str(&format!("&Tags[{}][Exclude]=false", i.to_string())),
					_ => continue,
				}
				i += 1;
			}
			// might remove this later in favor of the page listing
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int().unwrap_or(0) as i32;
				let option = match index {
					0 => "Newest",
					1 => "Oldest",
					2 => "Alpabetical",
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
	parameters.push_str(&helper::urlencode(page.to_string()));
	parameters.push_str(&query);
	parameters.push_str("&Sort=");
	parameters.push_str(&helper::urlencode(sort));
	parameters.push_str(&tags);
	let request = Request::new(&url, HttpMethod::Post)
		.header("User-Agent", "Aidoku")
		.body(format!("{}", parameters));
	let json = request.json()?.as_object()?;
	let data = json.get("data").as_array()?;

	let mut manga_arr: Vec<Manga> = Vec::new();
	for manga in data {
		let obj = manga.as_object()?;
		if let Ok(manga) = helper::parse_list(obj) {
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
	parameters.push_str(&helper::urlencode(page.to_string()));
	parameters.push_str("&Sort=");
	parameters.push_str(&helper::urlencode(listing.name));
	let request = Request::new(&url, HttpMethod::Post)
		.header("User-Agent", "Aidoku")
		.body(format!("{}", parameters));
	let json = request.json()?.as_object()?;
	let data = json.get("data").as_array()?;

	let mut manga_arr: Vec<Manga> = Vec::new();
	for manga in data {
		let obj = manga.as_object()?;
		if let Ok(manga) = helper::parse_list(obj) {
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
	let manga = helper::parse_manga(html)?;
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
	/* I dunno how much this is needed, as its the uploader name and theres usually only a few guys who upload
	let scanlator = html
		.select("div.book-info-container")
		.select("#Uploader")
		.select("a")
		.array()
		.get(0)
		.as_node()
		.expect("Failed to get uploader")
		.text()
		.read(); */

	Ok(vec![Chapter {
		id,
		title: String::from("Chapter 1"),
		volume: -1.0,
		chapter: 1.0,
		date_updated: date_uploaded,
		url,
		lang: String::from("en"),
		..Default::default()
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
	if num_pages.is_some() {
		for i in 1..(num_pages.unwrap_or(0) + 1) {
			let url = html
				.select("#image-container")
				.attr("data-cdn")
				.read()
				.replace("[PAGE]", &i.to_string());
			pages.push(Page {
				index: i,
				url,
				..Default::default()
			});
		}
		Ok(pages)
	} else {
		Err(aidoku::error::AidokuError {
			reason: aidoku::error::AidokuErrorKind::NodeError(aidoku::error::NodeError::ParseError),
		})
	}
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let numbers: Vec<&str> = url.rsplitn(2, '/').collect();
	let id = numbers[0].parse::<i32>().ok();
	match id {
		Some(id) => {
			let manga = get_manga_details(id.to_string());
			Ok(DeepLink {
				manga: Some(manga?),
				chapter: None,
			})
		}
		None => Err(aidoku::error::AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		}),
	}
}
