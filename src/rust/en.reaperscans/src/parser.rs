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

pub fn parse_chapter_list(base_url: String, id: String) -> Result<Vec<Chapter>> {
	let url = get_manga_url(id.clone(), base_url);

	let html = Request::new(&url, HttpMethod::Get)
		.html()
		.expect("Failed to get html");

	let mut chapters: Vec<Chapter> = Vec::new();

	for chapter in html.select("main div[wire:id] div > ul > li").array() {
		let chapter_node = chapter.as_node().expect("Failed to get chapter node");

		let mut title = String::new();
		let mut chapter_number = -1.0;

		let parsed_title = chapter_node
			.select("div.min-w-0 div.text-sm p.font-medium")
			.text()
			.read();

		// Only some titles have a chapter titles if they do
		// they are in the format of "Chapter 1 - Chapter Title" else
		// it's just "Chapter 1"
		if parsed_title.contains('-') {
			title = String::from(parsed_title.split('-').collect::<Vec<&str>>()[1].trim());
			chapter_number = parsed_title
				.replace("Chapter", "")
				.split('-')
				.collect::<Vec<&str>>()[0]
				.trim()
				.parse::<f32>()
				.expect("Failed to parse chapter number");
		} else {
			chapter_number = parsed_title
				.replace("Chapter", "")
				.trim()
				.parse::<f32>()
				.expect("Failed to parse chapter number");
		}

		let chapter_id = get_chapter_id(chapter_node.select("a").attr("href").read());
		let chapter_url = chapter_node.select("a").attr("href").read();

		let date_updated = get_date(
			chapter_node
				.select("div.min-w-0 div.text-xs p")
				.text()
				.read(),
		);

		chapters.push(Chapter {
			id: chapter_id,
			title,
			volume: -1.0,
			chapter: chapter_number,
			date_updated,
			scanlator: String::new(),
			url: chapter_url,
			lang: String::from("en"),
		});
	}
	Ok(chapters)
}

pub fn parse_page_list(
	base_url: String,
	manga_id: String,
	chapter_id: String,
) -> Result<Vec<Page>> {
	let url = get_chapter_url(chapter_id, manga_id, base_url);

	let html = Request::new(&url, HttpMethod::Get)
		.html()
		.expect("Failed to get html");

	let mut pages: Vec<Page> = Vec::new();

	for page in html.select("main div img.max-w-full").array() {
		let page_node = page.as_node().expect("Failed to get page node");

		let url = page_node.attr("src").read();

		let image_name = url
			.split('/')
			.last()
			.expect("Failed to get image name from url")
			.split('.')
			.next()
			.expect("Failed to get image name from url");

		let index = *extract_f32_from_string(String::from(image_name))
			.first()
			.expect("Failed to get index") as i32;

		let encoded_image_name = urlencode(String::from(image_name));
		let encoded_url = url.replace(&image_name, &encoded_image_name);

		pages.push(Page {
			index,
			url: encoded_url,
			base64: String::new(),
			text: String::new(),
		});
	}

	Ok(pages)
}
