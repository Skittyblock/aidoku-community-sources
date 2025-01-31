#![no_std]
extern crate alloc;
use aidoku::{
	error::Result,
	prelude::*,
	std::{html::Node, net::Request, String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};
use alloc::vec;
use manga_chan_template::template::{cache_manga_page, MangaChanSource, CACHED_MANGA};

static INSTANCE: MangaChanSource = MangaChanSource {
	base_url: "https://hentaichan.live",
	vol_chap_parser: |_, _| (-1.0, -1.0),
	author_selector: "div.row:contains(Автор) div.item2 a",
	custom_new_path: Some("manga"),
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
	cache_manga_page(format!("{}{id}", INSTANCE.base_url).as_str());
	let html = Node::new(unsafe { &CACHED_MANGA.clone().unwrap() })?;
	let date_updated = html
		.select("div.row4_right:contains(загружено) b")
		.text()
		.0
		.as_date("dd MMMM yyyy", Some("ru_RU"), None)
		.unwrap_or(-1.0);
	Ok(vec![Chapter {
		id: html.select("a:contains(Читать онлайн)").attr("href").read(),
		title: String::new(),
		volume: -1.0,
		chapter: 1.0,
		date_updated,
		scanlator: String::new(),
		url: html.select("a:contains(Читать онлайн)").attr("href").read(),
		lang: String::from("ru"),
	}])
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	INSTANCE.get_page_list(chapter_id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	INSTANCE.modify_image_request(request)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	INSTANCE.handle_url(url)
}
