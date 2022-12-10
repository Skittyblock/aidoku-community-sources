#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::HttpMethod, net::Request, print, String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer,
};

mod helper;

const BASE_URL: &str = "https://reaperscans.com";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let url = format!("{}/comics?page={}", BASE_URL, page);
	let html = Request::new(&url, HttpMethod::Get)
		.html()
		.expect("i brokey");
	let mut mangas: Vec<Manga> = Vec::new();
	for manga in html.select("main div[wire:id] div > li > div").array() {
		let manga_node = manga.as_node().expect("now i brokey");
		let title = String::from(manga_node.select("a.text-sm").text().read().trim());
		let id = manga_node.select("a").attr("href").read();
		let cover = manga_node.select("img").attr("src").read();
		// let id = get_manga_id(manga_node.select("a").attr("href").read());
		// let cover = base_url.clone() + &get_image_src(manga_node);
		mangas.push(Manga {
			id,
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: String::new(),
			categories: Vec::new(),
			status: MangaStatus::Unknown,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll,
		});
	}

	let last_page = html.select("main div[wire:id] div nav span").text().read();
	let has_more = last_page.contains("Next");

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	todo!()
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	todo!()
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	todo!()
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	todo!()
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	todo!()
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	todo!()
}
