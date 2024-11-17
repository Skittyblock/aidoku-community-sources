#![no_std]

mod helper;

use aidoku::{
	error::Result,
	helpers::substring::Substring,
	helpers::uri::encode_uri_component,
	prelude::*,
	std::net::{HttpMethod, Request},
	std::{String, StringRef, Vec},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

use helper::*;

const BASE_URL: &str = "https://asuracomic.net";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut url = format!("{}/series?page={}", BASE_URL, page);

	let mut genres = Vec::new();

	// '-1' means 'All', its's the default value for generes, status, and types
	// 'update' is the default value for order
	// All the filters are returned as JSON from this endpoint:
	// https://gg.asuracomic.net/api/series/filters
	// In the future source api rewrite, we can utilize this endpoint to dynamically
	// set the filters and their values.
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(value) = filter.value.as_string() {
					let query = encode_uri_component(value.read());
					url.push_str(format!("&name={query}").as_str());
				}
			}
			FilterType::Genre => {
				if let Ok(id) = filter.object.get("id").as_string() {
					match filter.value.as_int().unwrap_or(-1) {
						1 => genres.push(id.read()),
						_ => continue,
					}
				}
			}
			FilterType::Select => match filter.name.as_str() {
				"Status" => match filter.value.as_int().unwrap_or(-1) {
					1 => url.push_str("&status=1"),
					2 => url.push_str("&status=2"),
					3 => url.push_str("&status=3"),
					4 => url.push_str("&status=4"),
					5 => url.push_str("&status=5"),
					6 => url.push_str("&status=6"),
					_ => url.push_str("&status=-1"),
				},
				"Type" => match filter.value.as_int().unwrap_or(-1) {
					1 => url.push_str("&types=1"),
					2 => url.push_str("&types=2"),
					3 => url.push_str("&types=3"),
					_ => url.push_str("&types=-1"),
				},
				_ => continue,
			},
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int().unwrap_or(-1);
				match index {
					0 => url.push_str("&order=rating"),
					1 => url.push_str("&order=update"),
					2 => url.push_str("&order=latest"),
					3 => url.push_str("&order=desc"),
					4 => url.push_str("&order=asc"),
					_ => url.push_str("&order=update"),
				}
			}
			_ => continue,
		}
	}

	if !genres.is_empty() {
		url.push_str("&genres=");
		url.push_str(&genres.join(","));
	} else {
		url.push_str("&genres=-1");
	}

	let html = Request::new(&url, HttpMethod::Get).html()?;

	let mut manga: Vec<Manga> = Vec::new();

	for node in html.select("div.grid > a[href]").array() {
		let node = node.as_node()?;

		let raw_url = node.attr("abs:href").read();

		let id = get_manga_id(&raw_url)?;
		let url = get_manga_url(&id);

		let cover = node.select("img").attr("abs:src").read();
		let title = node.select("div.block > span.block").text().read();

		manga.push(Manga {
			id,
			cover,
			title,
			url,
			..Default::default()
		});
	}

	let has_more = !html
		.select("div.flex > a.flex.bg-themecolor:contains(Next)")
		.array()
		.is_empty();

	Ok(MangaPageResult { manga, has_more })
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = get_manga_url(&manga_id);

	let html = Request::new(&url, HttpMethod::Get).html()?;

	let wrapper = html.select("div.relative.grid");

	let cover = wrapper.select("img[alt=poster]").attr("abs:src").read();
	let title = wrapper.select("span.text-xl.font-bold").text().read();
	let author = {
		let text = wrapper
			.select("div:has(h3:eq(0):containsOwn(Author)) > h3:eq(1)")
			.text()
			.read();

		if text != "_" {
			text
		} else {
			String::new()
		}
	};

	let artist = {
		let text = wrapper
			.select("div:has(h3:eq(0):containsOwn(Artist)) > h3:eq(1)")
			.text()
			.read();

		if text != "_" {
			text
		} else {
			String::new()
		}
	};

	let description = wrapper.select("span.font-medium.text-sm").text().read();

	let mut categories = Vec::new();

	let mut nsfw = MangaContentRating::Safe;

	for genre in wrapper
		.select("div[class^=space] > div.flex > button.text-white")
		.array()
	{
		let genre = genre.as_node()?;
		let genre = genre.text().read();

		if genre == "Adult" || genre == "Ecchi" {
			nsfw = MangaContentRating::Suggestive;
		}

		categories.push(genre);
	}

	let status = {
		let status_string = wrapper
			.select("div.flex:has(h3:eq(0):containsOwn(Status)) > h3:eq(1)")
			.text()
			.read();

		match status_string.as_str() {
			"Ongoing" => MangaStatus::Ongoing,
			"Hiatus" => MangaStatus::Hiatus,
			"Completed" => MangaStatus::Completed,
			"Dropped" => MangaStatus::Cancelled,
			"Season End" => MangaStatus::Hiatus,
			_ => MangaStatus::Unknown,
		}
	};

	let viewer = {
		let format = wrapper
			.select("div.flex:has(h3:eq(0):containsOwn(Type)) > h3:eq(1)")
			.text()
			.read();

		match format.as_str() {
			"Manhwa" => MangaViewer::Scroll,
			"Manhua" => MangaViewer::Scroll,
			"Manga" => MangaViewer::Rtl,
			_ => MangaViewer::Scroll,
		}
	};

	Ok(Manga {
		id: manga_id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories,
		status,
		nsfw,
		viewer,
	})
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = get_manga_url(&manga_id);

	let html = Request::new(url, HttpMethod::Get).html()?;

	let mut chapters: Vec<Chapter> = Vec::new();

	for node in html
		.select("div.scrollbar-thumb-themecolor > div.group")
		.array()
	{
		let node = node.as_node()?;

		let raw_url = node.select("a").attr("abs:href").read();

		let id = get_chapter_id(&raw_url)?;
		let manga_id = get_manga_id(&raw_url)?;

		let url = get_chapter_url(&id, &manga_id);

		// Chapter's title if it exists
		let title = node.select("h3 > a > span").text().read();

		let chapter = node
			.select("h3 > a")
			.text()
			.read()
			.replace(&title, "")
			.replace("Chapter", "")
			.trim()
			.parse::<f32>()
			.unwrap_or(-1.0);

		let cleaned_date: StringRef = {
			let mut date = node.select("h3:not(:has(*))").text().read();

			let mut parts = date.split_whitespace().collect::<Vec<&str>>();

			// Check if the date has 3 parts, Month Day Year
			if parts.len() == 3 {
				let day = parts[1];

				// Remove any non-digit characters from the day
				// We are trying to remove all the suffixes from the day
				let cleaned_day = day
					.chars()
					.filter(|c| c.is_ascii_digit())
					.collect::<String>();

				parts[1] = &cleaned_day;

				date = parts.join(" ");
			}

			date.into()
		};

		let date_updated = cleaned_date.as_date("MMMM d yyyy", Some("en-US"), None);

		chapters.push(Chapter {
			id,
			title,
			chapter,
			date_updated,
			url,
			..Default::default()
		});
	}

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = get_chapter_url(&chapter_id, &manga_id);

	let html = Request::new(url, HttpMethod::Get).html()?;

	let mut pages: Vec<Page> = Vec::new();

	for node in html.select("div > img[alt^=chapter page]").array() {
		let node = node.as_node()?;

		let url = node.attr("abs:src").read();
		let index = {
			let before = url.substring_after_last('/').unwrap_or("");
			let after = before.substring_before('.').unwrap_or("");

			let cleaned_after = after
				.chars()
				.filter(|c| c.is_ascii_digit())
				.collect::<String>();

			cleaned_after.parse::<i32>().unwrap_or(-1)
		};

		pages.push(Page {
			index,
			url,
			..Default::default()
		});
	}

	Ok(pages)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.header("Referrer", BASE_URL);
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let manga_id = get_manga_id(&url)?;
	let chapter_id = get_chapter_id(&url)?;

	Ok(DeepLink {
		manga: get_manga_details(manga_id).ok(),
		chapter: Some(Chapter {
			id: chapter_id,
			..Default::default()
		}),
	})
}
