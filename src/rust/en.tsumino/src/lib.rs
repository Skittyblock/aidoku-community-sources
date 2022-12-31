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
use alloc::string::ToString;
mod helper;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	aidoku::prelude::println!("here");
	let mut sort = String::from("Newest");
	let mut tags: String = String::new();
	let mut i = 0;
	for filter in filters {
		match filter.kind {
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
	parameters.push_str("&Sort=");
	parameters.push_str(&helper::urlencode(sort));
	parameters.push_str(&tags);
	aidoku::prelude::println!("url: {}", url);
	aidoku::prelude::println!("parameters: {}", parameters);
	let request = Request::new(&url, HttpMethod::Post)
		.header("User-Agent", "Aidoku")
		.body(format!("{}", parameters));
	let json = request.json()?.as_object()?;
	let data = json.get("data").as_array()?;

	let mut manga_arr: Vec<Manga> = Vec::new();
	let total: i32;
	for manga in data {
		let obj = manga.as_object()?;
		if let Ok(manga) = helper::parse_manga(obj) {
			manga_arr.push(manga);
		}
	}
	total = json.get("pageCount").as_int().unwrap_or(0) as i32;

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
	let total: i32;
	for manga in data {
		let obj = manga.as_object()?;
		if let Ok(manga) = helper::parse_manga(obj) {
			manga_arr.push(manga);
		}
	}
	total = json.get("pageCount").as_int().unwrap_or(0) as i32;

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: page < total,
	})
}

#[get_manga_details]
fn get_manga_details(_: String) -> Result<Manga> {
	todo!()
}

#[get_chapter_list]
fn get_chapter_list(_: String) -> Result<Vec<Chapter>> {
	todo!()
}

#[get_page_list]
fn get_page_list(_: String, _: String) -> Result<Vec<Page>> {
	todo!()
}

// #[modify_image_request]
// fn modify_image_request(_: Request) {
// 	todo!()
// }

#[handle_url]
fn handle_url(_: String) -> Result<DeepLink> {
	todo!()
}
