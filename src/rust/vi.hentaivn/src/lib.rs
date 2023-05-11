#![no_std]
#![feature(let_chains)]
extern crate alloc;
mod parser;
mod search;

use aidoku::{
	error::{AidokuError, Result},
	helpers::uri::{encode_uri, QueryParameters},
	prelude::*,
	std::{defaults::defaults_get, net::Request, String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};
use alloc::string::ToString;
use parser::{
	parse_chapter_list, parse_manga_details, parse_new_or_complete_page, parse_page_list,
	parse_search_page,
};
use search::get_search_url;

pub static BASE_URL: &str = "https://hentaivn.tv";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let search_url = get_search_url(filters, page);
	let req = Request::get(&search_url).header("Referer", BASE_URL);
	parse_search_page(req.html()?, search_url.contains("tag%5B%5D=201"))
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let page_url = match listing.name.as_str() {
		"Chương mới" => "chap-moi.html",
		"Đã hoàn thành" => "da-hoan-thanh.html",
		_ => {
			return Err(AidokuError {
				reason: aidoku::error::AidokuErrorKind::Unimplemented,
			})
		}
	};
	let req =
		Request::get(format!("{BASE_URL}/{page_url}?page={page}")).header("Referer", BASE_URL);
	parse_new_or_complete_page(req.html()?)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = encode_uri(format!("{BASE_URL}/{id}"));
	let req = Request::get(url).header("Referer", BASE_URL);
	parse_manga_details(id, req.html()?)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let id_split = id.split("-doc-truyen-").collect::<Vec<_>>();

	let mut qs = QueryParameters::new();
	qs.push("idchapshow", Some(id_split[0]));
	qs.push("idlinkanime", Some(&id_split[1].replace(".html", "")));

	let req =
		Request::get(format!("{BASE_URL}/list-showchapter.php?{qs}")).header("Referer", BASE_URL);
	parse_chapter_list(req.html()?)
}

#[get_page_list]
fn get_page_list(_: String, id: String) -> Result<Vec<Page>> {
	let page_quality_ref = defaults_get("pageQuality")?;
	let page_quality = page_quality_ref.as_string()?.read();

	let url = encode_uri(format!("{BASE_URL}/{id}"));
	if page_quality == "1200" {
		let req = Request::get(url)
			.header("Referer", BASE_URL)
			.header("Cookie", "view1=1");
		parse_page_list(req.html()?, None)
	} else {
		let numeric_id = id.split('-').collect::<Vec<_>>()[1];

		let server_type = match page_quality.as_str() {
			"9999" => "3",
			"800" => "1",
			"600" => "2",
			_ => "1",
		};
		let req = Request::post(format!("{BASE_URL}/ajax_load_server.php"))
			.header("Referer", &url)
			.header("Content-Type", "application/x-www-form-urlencoded")
			.header("Cookie", "view1=1")
			.body(format!("server_id={numeric_id}&server_type={server_type}"));
		parse_page_list(req.html()?, Some("img"))
	}
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.header("Referer", BASE_URL);
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let manga_or_chapter_id = url
		.split('/')
		.last()
		.expect("handle_url expected last element");
	if manga_or_chapter_id.contains("doc-truyen") {
		Ok(DeepLink {
			manga: get_manga_details(manga_or_chapter_id.to_string()).ok(),
			chapter: None,
		})
	} else {
		Err(AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		})
	}
}
