use aidoku::{
	error::Result,
	helpers::uri::encode_uri_component,
	prelude::format,
	std::html::Node,
	std::net::{HttpMethod, Request},
	std::{String, Vec},
	Chapter, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page,
};

extern crate alloc;
use alloc::string::ToString;

use crate::helper::*;
use crate::request_helper::*;

pub fn parse_manga_list(base_url: String, page: i32) -> Result<MangaPageResult> {
	let url = format!("{}/comics?page={}", base_url, page);

	let html = Request::new(url, HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.html()
		.expect(
		"ReaperScans: Could not display All listing. Check the website and your internet connection on https://reaperscans.com",
		);

	let mut mangas: Vec<Manga> = Vec::new();

	for manga in html.select("main div[wire:id] div > li > div").array() {
		let manga_node = manga
			.as_node()
			.expect("Reaperscans: Failed to parse a manga node. Title could not be displayed");
		let id = get_manga_id(manga_node.select("a").attr("href").read());
		let cover = manga_node.select("img").attr("src").read();
		let title = String::from(manga_node.select("a:not(:has(img))").text().read().trim());
		let url = manga_node.select("a").attr("href").read();

		mangas.push(Manga {
			id,
			cover,
			title,
			url,
			viewer: MangaViewer::Scroll,
			..Default::default()
		});
	}

	let pagination = html.select("main div[wire:id] div nav span").text().read();
	let has_more = pagination.contains("Next");

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_search(base_url: String, query: String) -> Result<MangaPageResult> {
	let html: Node = create_search_request_object(String::from(&base_url), query)
		.expect("Reaperscans: Search POST request was unsuccessful");

	let mut mangas: Vec<Manga> = Vec::new();
	for i in html.select("ul li").array() {
		let item = i
			.as_node()
			.expect("Reaperscans: Failed to parse a manga node. Title could not be displayed");
		// Search displays both Comics and Novels in that order.
		if &item.text().read() == "Novels" {
			break;
		}
		let id = item
			.select("a")
			.attr("href")
			.read()
			.replace(&base_url, "")
			.replace("/comics/", "");
		if id.is_empty() {
			continue;
		}
		let cover = item.select("a img").attr("src").read();
		let title = item.select("a img").attr("alt").read();
		mangas.push(Manga {
			id,
			cover,
			title,
			viewer: MangaViewer::Scroll,
			..Default::default()
		});
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more: false,
	})
}

pub fn parse_manga_listing(
	base_url: String,
	_listing: Listing,
	page: i32,
) -> Result<MangaPageResult> {
	// The only alternate listing that reaper has is latest
	let url = format!("{}/latest/comics?page={}", base_url, page);

	let html = Request::new(url, HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.html()
		.expect("ReaperScans: Could not display a listing. Check the website and your internet connection on https://reaperscans.com");

	let mut mangas: Vec<Manga> = Vec::new();

	for manga in html
		.select("main div[wire:id] > div > div:not(:has(nav)) > div")
		.array()
	{
		let manga_node = manga
			.as_node()
			.expect("Reaperscans: Failed to parse a manga node. Title could not be displayed");
		let id = get_manga_id(manga_node.select("a").attr("href").read());
		let cover = manga_node.select("img").attr("src").read();
		let title = String::from(manga_node.select("p > a").text().read().trim());
		let url = manga_node.select("a").attr("href").read();

		mangas.push(Manga {
			id,
			cover,
			title,
			url,
			viewer: MangaViewer::Scroll,
			..Default::default()
		});
	}

	let pagination = html.select("main div[wire:id] div nav span").text().read();
	let has_more = pagination.contains("Next");

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_manga_details(base_url: String, manga_id: String) -> Result<Manga> {
	let url = get_manga_url(manga_id.clone(), base_url);

	let html = Request::new(&url, HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.html()
		.expect("Reaperscans: Failed to access the comic page. Try accessing the website on a browser and check your internet connection");

	let main_div = html.select("main > div:not(:has(script))");

	let cover = main_div.select("div[tabindex] img[alt]").attr("src").read();

	let title = String::from(
		main_div
			.select("div[tabindex] div:not(:has(dl)) h1")
			.text()
			.read()
			.trim(),
	);

	let description = text_with_newlines(main_div.select("section div[tabindex] > div > p"));

	let mut status = MangaStatus::Unknown;
	let mut age_rating = String::new();
	let mut nsfw = MangaContentRating::Safe;
	let mut language = String::new();

	for node in main_div
		.select("section div[tabindex] > div dl > div")
		.array()
	{
		let info = node
			.as_node()
			.expect("Reaperscans: Could not parse the comic's details")
			.text()
			.read();

		if info.contains("Source Language") {
			language = String::from(info.replace("Source Language", "").trim());
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
		_ => MangaViewer::Scroll,
	};

	Ok(Manga {
		id: manga_id,
		cover,
		title,
		description,
		url,
		status,
		nsfw,
		viewer,
		..Default::default()
	})
}

pub fn parse_chapter_list(base_url: String, manga_id: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	let url = get_manga_url(manga_id, base_url.clone());
	let request = Request::new(url, HttpMethod::Get).header("User-Agent", USER_AGENT);

	let initial_request = request
		.html()
		.expect("Reaperscans: Could not retreive the chapter list html");

	parse_chapter_list_helper(initial_request.clone(), &mut chapters);

	let mut page = 2;

	let max_retries = 5;
	let mut retries = 0;

	while retries < max_retries {
		// If let here gracefully handles the case where cloudflare blocks the request.
		// If the request is blocked, then the loop will continue and try again, with
		// a maximum of 5 retries.
		// The cloudflare popup should appear and once validated, the next request will
		// be successful.
		if let Ok(response_html) = create_chapter_request_object(
			initial_request.clone(),
			base_url.clone(),
			page.to_string(),
		) {
			parse_chapter_list_helper(response_html.clone(), &mut chapters);

			// checks if next-page button exists.
			if response_html
				.select("button[wire:click*=nextPage]")
				.html()
				.read()
				.is_empty()
			{
				break;
			}

			retries = 0;
			page += 1;
		} else {
			retries += 1;
			continue;
		}
	}
	Ok(chapters)
}

pub fn parse_chapter_list_helper(html: Node, chapters: &mut Vec<Chapter>) {
	for chapter in html.select("div[wire:id] div > ul > li").array() {
		let chapter_node = chapter.as_node().expect(
			"Reaperscans: Failed to parse a chapter node. The chapter list request was unsuccessful",
		);

		let mut title = String::new();
		let chapter_number;

		let parsed_title = chapter_node.select("div div p[class]").text().read();

		// Only some titles have a chapter titles if they do
		// they are in the format of "Chapter 1 - Chapter Title" else
		// it's just "Chapter 1"
		if parsed_title.contains('-') {
			title = String::from(
				parsed_title
					.split('-')
					.last()
					.expect("Reaperscans: Failed to get chapter title")
					.trim(),
			);
			chapter_number = parsed_title
				.replace("Chapter", "")
				.split('-')
				.next()
				.expect("Reaperscans: Failed to get chapter number")
				.trim()
				.parse::<f32>()
				.expect("Reaperscans: Failed to parse chapter number");
		} else {
			chapter_number = parsed_title
				.replace("Chapter", "")
				.trim()
				.parse::<f32>()
				.expect("Reaperscans: Failed to parse chapter number");
		}

		let chapter_id = get_chapter_id(chapter_node.select("a").attr("href").read());
		let chapter_url = chapter_node.select("a").attr("href").read();

		let date_updated = get_date(chapter_node.select("div div p:not([class])").text().read());

		chapters.push(Chapter {
			id: chapter_id,
			title,
			chapter: chapter_number,
			date_updated,
			url: chapter_url,
			lang: String::from("en"),
			..Default::default()
		});
	}
}

pub fn parse_page_list(
	base_url: String,
	manga_id: String,
	chapter_id: String,
) -> Result<Vec<Page>> {
	let url = get_chapter_url(chapter_id, manga_id, base_url);

	let html = Request::new(url, HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.html()
		.expect("Reaperscans: Failed to display the chapter. Check your internet connection");

	let mut pages: Vec<Page> = Vec::new();
	// Stores the indices of the pages so duplicates can be incremented.
	let mut indices: Vec<i32> = Vec::new();

	// Select all images that are not children of a noscript tag.
	for page in html.select("main div > img:not(noscript *)").array() {
		let page_node = page
			.as_node()
			.expect("Reaperscans: Failed to parse a chapter node. The chapter pages request was unsuccessful");

		let url = {
			let src = page_node.attr("src").read();
			let data_cfsrc = page_node.attr("data-cfsrc").read();

			if !src.is_empty() {
				src
			} else if !data_cfsrc.is_empty() {
				data_cfsrc
			} else {
				panic!("Reaperscans: Failed to get the image url");
			}
		};

		let image_name = url
			.split('/')
			.last()
			.expect("Reaperscans: Could not get the image url for the chapter")
			.split('.')
			.next()
			.expect("Reaperscans: Could not get the image url for the chapter");

		let mut index = *extract_f32_from_string(String::from(image_name))
			.first()
			.expect("Reaperscans: Failed to get index") as i32;

		// ReaperScans sometimes has duplicate image numbers, so this will increment the
		// index for pages that have the same index.
		if indices.contains(&index) {
			index += 1;
		}
		indices.push(index);

		let encoded_image_name = encode_uri_component(String::from(image_name));
		let encoded_url = url.replace(image_name, &encoded_image_name);

		pages.push(Page {
			index,
			url: encoded_url,
			..Default::default()
		});
	}

	Ok(pages)
}

pub fn modify_image_request(base_url: String, request: Request) {
	request.header("Referer", &base_url);
}
