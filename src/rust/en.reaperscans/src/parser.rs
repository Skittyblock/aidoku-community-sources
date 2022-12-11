#![no_std]
use aidoku::{
	error::Result,
	prelude::format,
	std::net::HttpMethod,
	std::net::Request,
	std::Vec,
	std::{print, String},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

use crate::helper::*;

pub fn parse_manga_list(
	base_url: String,
	filters: Vec<Filter>,
	page: i32,
) -> Result<MangaPageResult> {
	let url = format!("{}/comics?page={}", base_url, page);

	let html = Request::new(url, HttpMethod::Get)
		.html()
		.expect("Failed to get html");

	let mut mangas: Vec<Manga> = Vec::new();

	for manga in html.select("main div[wire:id] div > li > div").array() {
		let manga_node = manga.as_node().expect("Failed to get manga node");
		let id = get_manga_id(manga_node.select("a").attr("href").read());
		let cover = manga_node.select("img").attr("src").read();
		let title = String::from(manga_node.select("a.text-sm").text().read().trim());
		let url = manga_node.select("a").attr("href").read();

		mangas.push(Manga {
			id,
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url,
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
