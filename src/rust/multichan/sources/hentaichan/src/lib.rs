#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec, html::Node},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

use manga_chan_template::template::{MangaChanSource, CACHED_MANGA, cache_manga_page};

static INSTANCE: MangaChanSource = MangaChanSource {
	base_url: "https://y.hentaichan.live",
	vol_chap_parser: |_, _| (-1.0, -1.0),
	author_selector: "div.row:contains(Автор) div.item2 a",
};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	INSTANCE.get_manga_list(filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	INSTANCE.get_manga_listing(listing, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	INSTANCE.get_manga_details(id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	cache_manga_page(
		format!("{}{id}", INSTANCE.base_url).as_str()
	);
	let html = Node::new(unsafe { &CACHED_MANGA.clone().unwrap() });
	let date_updated = html
		.select("div.row4_right:contains(загружено) b")
		.text()
		.0
		.as_date(
			"dd MMMM yyyy",
			Some("ru_RU"),
			None
		)
		.unwrap_or(-1.0);
	let mut chapters = Vec::with_capacity(1);
	chapters.push(Chapter { 
		id: html.select("a:contains(Читать онлайн)").attr("href").read(), 
		title: String::new(),
		volume: -1.0, 
		chapter: 1.0, 
		date_updated, 
		scanlator: String::new(), 
		url: html.select("a:contains(Читать онлайн)").attr("href").read(), 
		lang: String::from("ru"),
	});
	Ok(chapters)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	INSTANCE.get_page_list(id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	INSTANCE.modify_image_request(request)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	INSTANCE.handle_url(url)
}
