#![no_std]

use aidoku::{
	error::Result,
	helpers::uri::QueryParameters,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

mod helper;
mod model;

use helper::*;
use model::SortOptions;

const BASE_URL: &str = "https://weebcentral.com";

const FETCH_LIMIT: i32 = 24;

// don't really need listings for now
// #[get_manga_listing]
// fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
// 	Err(aidoku::error::AidokuError {
// 		reason: aidoku::error::AidokuErrorKind::Unimplemented,
// 	})
// }

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut qs = QueryParameters::new();
	qs.push("display_mode", Some("Full Display"));

	let mut extra_params = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				let title = remove_special_chars(filter.value.as_string()?.read());
				let query = title.trim();
				qs.push("text", Some(query));
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let idx = value.get("index").as_int().unwrap_or(0) as i32;
				let ascending = value.get("ascending").as_bool().unwrap_or(false);
				let option = SortOptions::from(idx);

				qs.push("sort", Some(option.into()));
				qs.push(
					"order",
					Some(if ascending { "Ascending" } else { "Descending" }),
				);
			}
			FilterType::Check => {
				let value = filter.value.as_int().unwrap_or(-1);
				if value < 0 {
					continue;
				}
				let id = filter.object.get("id").as_string()?.read();

				if id == "official" {
					qs.push(
						"official",
						Some(match value {
							0 => "False",
							1 => "True",
							_ => "Any",
						}),
					);
				} else {
					extra_params.push_str(&id);
				}
			}
			FilterType::Genre => {
				let value = filter.value.as_int().unwrap_or(-1);
				if value < 0 {
					continue;
				}
				let Ok(id) = filter.object.get("name").as_string() else {
					continue;
				};
				qs.push(
					match value {
						0 => "excluded_tag",
						1 => "included_tag",
						_ => continue,
					},
					Some(&id.read()),
				);
			}
			_ => continue,
		}
	}

	let offset = (page - 1) * FETCH_LIMIT;

	let url = format!(
		"{BASE_URL}/search/data\
			?limit={FETCH_LIMIT}\
			&offset={offset}\
			&{qs}\
			{extra_params}"
	);

	let html = Request::get(&url).html()?;

	let mut manga = Vec::new();

	for element in html.select("article:has(section)").array() {
		let element = element.as_node().expect("html array will always be nodes");

		let cover = element.select("img").first().attr("abs:src").read();

		let title_element = element.select("a").first();
		let mut title = title_element.text().read();

		const OFFICIAL_PREFIX: &str = "Official ";
		if title.starts_with(OFFICIAL_PREFIX) {
			title = title[OFFICIAL_PREFIX.len()..].trim().into();
		}

		let url = title_element.attr("abs:href").read();

		let Some(id) = url.strip_prefix(BASE_URL).map(String::from) else {
			continue;
		};

		manga.push(Manga {
			id,
			title,
			cover,
			..Default::default()
		});
	}

	let has_more = !manga.is_empty();

	Ok(MangaPageResult { manga, has_more })
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{BASE_URL}{id}");
	let html = Request::get(&url).html()?;

	let elements = html.select("section[x-data] > section");

	let info_element = elements.first();
	let title_element = elements.last();

	let title = title_element.select("h1").first().text().read();

	let cover = info_element.select("img").first().attr("abs:src").read();

	let description = title_element
		.select("li:has(strong:contains(Description)) > p")
		.first()
		.text()
		.read();

	let author = info_element
		.select("ul > li:has(strong:contains(Author)) > span > a")
		.array()
		.map(|value| value.as_node().unwrap().text().read())
		.collect::<Vec<String>>()
		.join(", ");

	let categories = info_element
		.select("ul > li:has(strong:contains(Tag),strong:contains(Type)) a")
		.array()
		.map(|value| value.as_node().unwrap().text().read())
		.collect::<Vec<String>>();

	let status = match info_element
		.select("ul > li:has(strong:contains(Status)) a")
		.text()
		.read()
		.as_str()
	{
		"Complete" => MangaStatus::Completed,
		"Ongoing" => MangaStatus::Ongoing,
		"Hiatus" => MangaStatus::Hiatus,
		"Canceled" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	};

	let nsfw = if categories
		.iter()
		.any(|e| e == "Adult" || e == "Hentai" || e == "Mature")
	{
		MangaContentRating::Nsfw
	} else if categories.iter().any(|e| e == "Ecchi") {
		MangaContentRating::Suggestive
	} else {
		MangaContentRating::Safe
	};

	let viewer = match info_element
		.select("ul > li:has(strong:contains(Type)) a")
		.first()
		.text()
		.read()
		.as_str()
	{
		"Manhua" => MangaViewer::Scroll,
		"Manhwa" => MangaViewer::Scroll,
		_ => MangaViewer::Rtl,
	};

	Ok(Manga {
		id,
		title,
		cover,
		author,
		description,
		url,
		categories,
		status,
		nsfw,
		viewer,
		..Default::default()
	})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let manga_url = format!("{BASE_URL}{id}");

	let url = if let Some(last_slash_pos) = manga_url.rfind('/') {
		let base_url = &manga_url[..last_slash_pos];
		format!("{}/full-chapter-list", base_url)
	} else {
		manga_url
	};
	let html = Request::get(&url).html()?;

	let mut chapters = Vec::new();

	for element in html.select("div[x-data]").array() {
		let element = element.as_node().expect("html array will always be nodes");

		let mut title = element.select("span.flex > span").first().text().read();
		let url = element.select("a").first().attr("abs:href").read();

		let Some(chapter_id) = url.strip_prefix(BASE_URL).map(String::from) else {
			continue;
		};

		let mut chapter = if let Some(last_space) = title.rfind(' ') {
			let chapter_number_text = &title[last_space + 1..];
			chapter_number_text.parse::<f32>().unwrap_or(-1.0)
		} else {
			-1.0
		};

		let volume = if title.contains("Volume") {
			let volume = chapter;
			chapter = -1.0;
			title = String::default();
			volume
		} else {
			-1.0
		};

		if title.contains("Chapter") {
			title = String::default();
		}

		let date_updated = element.select("time[datetime]").attr("datetime").as_date(
			"yyyy-MM-dd'T'HH:mm:ss.SSS'Z'",
			Some("en-US"),
			None,
		);

		chapters.push(Chapter {
			id: chapter_id,
			title,
			volume,
			chapter,
			url,
			date_updated,
			..Default::default()
		});
	}

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{BASE_URL}{chapter_id}/images?is_prev=False&reading_style=long_strip");
	let html = Request::get(&url).html()?;

	let mut pages = Vec::new();

	for (idx, element) in html
		.select("section[x-data~=scroll] > img")
		.array()
		.enumerate()
	{
		let element = element.as_node().expect("html array will always be nodes");
		let page_url = element.attr("abs:src").read();
		pages.push(Page {
			index: idx as i32,
			url: page_url,
			..Default::default()
		});
	}

	Ok(pages)
}

// i don't care enough to implement this rn
// #[handle_url]
// fn handle_url(url: String) -> Result<DeepLink> {
// 	Err(aidoku::error::AidokuError {
// 		reason: aidoku::error::AidokuErrorKind::Unimplemented,
// 	})
// }
