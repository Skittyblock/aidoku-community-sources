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

// TODO: Add search support, reaper uses a rest api for searching that uses a weird url format that could change at any time
// need to figure out a good way to deal with that, or steal tachiyomi's implementation
// https://reaperscans.com/livewire/message/frontend.dtddzhx-ghvjlgrpt
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
	let url = match listing.name.as_str() {
		"Latest" => format!("{}/latest/comics?page={}", BASE_URL, page),
		_ => format!("{}/comics/{}", BASE_URL, page),
	};

	let mut mangas: Vec<Manga> = Vec::new();
	let mut has_more = false;

	if &listing.name == "Latest" {
		let html = Request::new(&url, HttpMethod::Get)
			.html()
			.expect("i brokey");
		for manga in html
			.select("main div[wire:id] div.grid > div.relative")
			.array()
		{
			let manga_node = manga.as_node().expect("now i brokey");
			let title = String::from(manga_node.select("p.text-sm a").text().read().trim());
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
		has_more = last_page.contains("Next");
	} else {
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
		has_more = last_page.contains("Next");
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	// let id = get_full_url(raw_id);
	let html = Request::new(id.as_str(), HttpMethod::Get)
		.html()
		.expect("i brokey");
	let title = String::from(
		html.select("main div.grid div.container h1")
			.text()
			.read()
			.trim(),
	);
	let cover = html
		.select("main div.grid div.container img")
		.attr("src")
		.read();
	let description = String::from(
		html.select("main div.grid section div > div.p-4 > p")
			.text()
			.read()
			.trim(),
	);

	let mut language = String::new();
	let mut age_rating = String::new();
	let mut status = MangaStatus::Unknown;

	for node in html
		.select("main div.grid section div > div.p-4 > div > dl > div")
		.array()
	{
		let data = node.as_node().expect("now i brokey").text().read();
		if data.contains("Source Language") {
			language = data.replace("Source Language", "");
		} else if data.contains("Age Rating") {
			age_rating = data.replace("Age Rating", "");
		} else if data.contains("Release Status") {
			let release_status = data.replace("Release Status", "");
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

	let mut nsfw = MangaContentRating::Safe;

	if age_rating.contains("18+") {
		nsfw = MangaContentRating::Nsfw;
	} else if age_rating.contains("16+") {
		nsfw = MangaContentRating::Suggestive;
	}

	let viewer = match language.as_str() {
		"Japanese" => MangaViewer::Rtl,
		"Korean" => MangaViewer::Scroll,
		_ => MangaViewer::Rtl,
	};

	Ok(Manga {
		id: id.clone(),
		cover,
		title,
		author: String::new(),
		artist: String::new(),
		description,
		url: id,
		categories: Vec::new(),
		status,
		nsfw,
		viewer,
	})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	let html = Request::new(id.as_str(), HttpMethod::Get)
		.html()
		.expect("i brokey");
	for chapter in html.select("main div[wire:id] div > ul > li").array() {
		let chapter_node = chapter.as_node().expect("now i brokey");
		let mut title = String::new();
		let parsed_title = chapter_node
			.select("div.min-w-0 div.text-sm p.font-medium")
			.text()
			.read();

		let mut chapter_number = -1.0;

		if parsed_title.contains('-') {
			title = String::from(parsed_title.split('-').collect::<Vec<&str>>()[1].trim());
			chapter_number = parsed_title
				.replace("Chapter", "")
				.split('-')
				.collect::<Vec<&str>>()[0]
				.trim()
				.parse::<f32>()
				.expect("i brokey");
		} else {
			chapter_number = parsed_title
				.replace("Chapter", "")
				.trim()
				.parse::<f32>()
				.expect("i brokey");
		}

		let chapter_id = chapter_node.select("a").attr("href").read();
		let chapter_url = chapter_node.select("a").attr("href").read();

		// let date = get_date(chapter_node.select(".episode-date").text().read());
		chapters.push(Chapter {
			id: chapter_id,
			title,
			volume: -1.0,
			chapter: chapter_number,
			date_updated: -1.0,
			scanlator: String::new(),
			url: chapter_url,
			lang: String::from("en"),
		});
	}
	Ok(chapters)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	todo!()
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	todo!()
}
