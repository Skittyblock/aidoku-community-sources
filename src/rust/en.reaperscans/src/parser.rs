#![no_std]
use aidoku::{
	error::Result,
	prelude::format,
	std::net::HttpMethod,
	std::net::Request,
	std::Vec,
	std::{print, String},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult,
	MangaStatus, MangaViewer, Page,
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

	let pagination = html.select("main div[wire:id] div nav span").text().read();
	let has_more = pagination.contains("Next");

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_manga_listing(
	base_url: String,
	listing: Listing,
	page: i32,
) -> Result<MangaPageResult> {
	// The only alternate listing that reaper has is latest
	let url = match listing.name.as_str() {
		_ => format!("{}/latest/comics?page={}", base_url, page),
	};

	let html = Request::new(&url, HttpMethod::Get)
		.html()
		.expect("Failed to get html");

	let mut mangas: Vec<Manga> = Vec::new();

	for manga in html
		.select("main div[wire:id] div.grid > div.relative")
		.array()
	{
		let manga_node = manga.as_node().expect("Failed to get manga node");
		let id = get_manga_id(manga_node.select("a").attr("href").read());
		let cover = manga_node.select("img").attr("src").read();
		let title = String::from(manga_node.select("p.text-sm a").text().read().trim());
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

	let pagination = html.select("main div[wire:id] div nav span").text().read();
	let has_more = pagination.contains("Next");

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_manga_details(base_url: String, id: String) -> Result<Manga> {
	let url = get_manga_url(id.clone(), base_url);

	let html = Request::new(&url, HttpMethod::Get)
		.html()
		.expect("Failed to get html");

	let cover = html
		.select("main div.grid div.container img")
		.attr("src")
		.read();
	let title = String::from(
		html.select("main div.grid div.container h1")
			.text()
			.read()
			.trim(),
	);
	let description = String::from(
		html.select("main div.grid section div > div.p-4 > p")
			.text()
			.read()
			.trim(),
	);

	let mut status = MangaStatus::Unknown;
	let mut age_rating = String::new();
	let mut nsfw = MangaContentRating::Safe;
	let mut language = String::new();

	for node in html
		.select("main div.grid section div > div.p-4 > div > dl > div")
		.array()
	{
		let info = node
			.as_node()
			.expect("Failed to get info node")
			.text()
			.read();

		if info.contains("Source Language") {
			language = info.replace("Source Language", "");
		} else if info.contains("Age Rating") {
			age_rating = info.replace("Age Rating", "");
		} else if info.contains("Release Status") {
			let release_status = info.replace("Release Status", "");

			if release_status.contains("Ongoing") {
				status = MangaStatus::Ongoing;
			} else if release_status.contains("Complete") {
				status = MangaStatus::Completed;
			} else if release_status.contains("Dropped") {
				status = MangaStatus::Cancelled;
			} else if release_status.contains("On hold") {
				status = MangaStatus::Hiatus;
			}
		}
	}

	if age_rating.contains("18+") {
		nsfw = MangaContentRating::Nsfw;
	} else if age_rating.contains("16+") {
		nsfw = MangaContentRating::Suggestive;
	}

	let viewer = match language.as_str() {
		"Japanese" => MangaViewer::Rtl,
		"Korean" => MangaViewer::Scroll,
		"Chinese" => MangaViewer::Scroll,
		_ => MangaViewer::Rtl,
	};

	Ok(Manga {
		id,
		cover,
		title,
		author: String::new(),
		artist: String::new(),
		description,
		url,
		categories: Vec::new(),
		status,
		nsfw,
		viewer,
	})
}
