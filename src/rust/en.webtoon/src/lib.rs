#![no_std]
extern crate alloc;

use aidoku::{
	error::Result,
	prelude::*,
	std::{
		net::{HttpMethod, Request},
		ObjectRef, String, ValueRef, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, Page,
};
use alloc::string::ToString;

mod parser;
use crate::parser::urlencode;

const BASE_URL: &str = "https://webtoons.com";
const MOBILE_BASE_URL: &str = "https://m.webtoons.com";
const MOBILE_USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 13_2_3 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/13.0.3 Mobile/15E148 Safari/604.1";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, _page: i32) -> Result<MangaPageResult> {
	let mut listing_index = 0;
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				let url = format!(
					"{}/en/search?keyword={}",
					BASE_URL,
					urlencode(filter.value.as_string()?.read())
				);
				let html = Request::new(url.as_str(), HttpMethod::Get).html();

				let mut manga = parser::parse_search(&html, false);
				manga.append(&mut parser::parse_search(&html, true));

				return Ok(MangaPageResult {
					manga,
					has_more: false,
				});
			}
			FilterType::Select => {
				if filter.name != "listing_index" {
					continue;
				}

				listing_index = filter.value.as_int().unwrap_or(0);
			}
			_ => continue,
		}
	}

	let url = format!("{}/en/top", BASE_URL);
	let html = Request::new(url.as_str(), HttpMethod::Get).html();

	let mut result: Vec<Manga> = Vec::new();
	match listing_index {
		1 => result.append(&mut parser::parse_manga_list_popular(&html)),
		2 => result.append(&mut parser::parse_manga_list_trending(&html)),
		_ => {
			result.append(&mut parser::parse_manga_list_popular(&html));
			result.append(&mut parser::parse_manga_list_trending(&html));
		}
	}

	Ok(MangaPageResult {
		manga: result,
		has_more: false,
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let mut filters: Vec<Filter> = Vec::with_capacity(1);
	let value = ValueRef::from(match listing.name.as_str() {
		"Popular" => 1,
		"Trending" => 2,
		_ => 0,
	});

	filters.push(Filter {
		kind: FilterType::Select,
		name: "listing_index".to_string(),
		object: ObjectRef::new(),
		value,
	});

	get_manga_list(filters, page)
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = format!("{}/en/{}", MOBILE_BASE_URL, &manga_id);
	let html = Request::new(url.as_str(), HttpMethod::Get)
		.header("User-Agent", MOBILE_USER_AGENT)
		.html();
	parser::parse_manga(html, manga_id)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/en/{}", MOBILE_BASE_URL, &manga_id);
	let html = Request::new(url.as_str(), HttpMethod::Get)
		.header("User-Agent", MOBILE_USER_AGENT)
		.html();

	parser::get_chapter_list(html, manga_id)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	let ids = id.splitn(2, '|').collect::<Vec<_>>();
	let url = format!("{}/en/{}&episode_no={}", BASE_URL, ids[0], ids[1])
		.replace("list", format!("ep{}/viewer", ids[1]).as_str());

	let html = Request::new(url.as_str(), HttpMethod::Get).html();
	parser::get_page_list(html)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
        .header("Cookie", "pagGDPR=true;")
        .header("Referer", format!("{}/", BASE_URL).as_str())
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.0.5005.124 Safari/537.36 Edg/102.0.1245.44");
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	parser::handle_url(url)
}
