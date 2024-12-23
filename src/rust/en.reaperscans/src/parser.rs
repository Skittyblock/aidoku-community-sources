use aidoku::{
	error::Result,
	helpers::uri::encode_uri_component,
	prelude::format,
	std::{
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, Filter, FilterType, Listing, Manga, MangaPageResult, MangaStatus, MangaViewer, Page,
};

use crate::BASE_API_URL;

extern crate alloc;
use alloc::string::ToString;

pub fn parse_manga_list(
	base_url: String,
	filters: Vec<Filter>,
	page: i32,
) -> Result<MangaPageResult> {
	let mut search_query = String::new();
	let mut genres = String::new();
	let mut status = String::new();
	let mut order_by = String::new();
	let mut order = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				search_query = encode_uri_component(filter.value.as_string()?.read());
			}
			FilterType::Genre => {
				if let Ok(filter_id) = filter.object.get("id").as_string() {
					match filter.value.as_int().unwrap_or(-1) {
						1 => {
							genres.push_str(filter_id.read().as_str());
							genres.push(',');
						}
						_ => continue,
					}
				}
			}
			FilterType::Sort => {
				if let Ok(value) = filter.value.as_object() {
					let index = value.get("index").as_int().unwrap_or(0);
					let asc = value.get("ascending").as_bool().unwrap_or(false);

					order.push_str(if asc { "asc" } else { "desc" });
					match index {
						0 => order_by.push_str("updated_at"),
						1 => order_by.push_str("total_views"),
						2 => order_by.push_str("title"),
						3 => order_by.push_str("created_at"),
						_ => order_by.push_str("updated_at"),
					}
				}
			}
			FilterType::Select => match filter.name.as_str() {
				"Status" => match filter.value.as_int().unwrap_or(-1) {
					0 => status.push_str("All"),
					1 => status.push_str("Ongoing"),
					2 => status.push_str("Hiatus"),
					3 => status.push_str("Dropped"),
					4 => status.push_str("Completed"),
					_ => status.push_str("All"),
				},
				_ => continue,
			},
			_ => continue,
		}
	}

	if !genres.is_empty() {
		genres.pop();
	}

	let mut url = format!(
		"{}/query?page={}&perPage=20&series_type=Comic&adult=true",
		BASE_API_URL, page
	);

	if !search_query.is_empty() {
		url.push_str(&format!("&query_string={}", search_query));
	}
	if !order.is_empty() {
		url.push_str(&format!("&order={}", order));
	}
	if !order_by.is_empty() {
		url.push_str(&format!("&orderBy={}", order_by));
	}
	if !status.is_empty() {
		url.push_str(&format!("&status={}", status));
	}
	if !genres.is_empty() {
		url.push_str(&format!("&tags_ids=[{}]", genres));
	}

	let json = Request::new(url, HttpMethod::Get);
	let manga = parse_manga(&base_url, json)?;
	let has_more = !manga.is_empty();

	Ok(MangaPageResult { manga, has_more })
}

pub fn parse_manga_listing(
	base_url: String,
	_listing: Listing,
	page: i32,
) -> Result<MangaPageResult> {
	let url = format!("{}/query?page={}&perPage=20&series_type=Comic&query_string=&order=desc&orderBy=&adult=true&status=&tags_ids=[]", BASE_API_URL, page);

	let json = Request::new(url, HttpMethod::Get);
	let manga = parse_manga(&base_url, json)?;
	let has_more = !manga.is_empty();

	Ok(MangaPageResult { manga, has_more })
}

pub fn parse_manga_details(base_url: &String, manga_id: String) -> Result<Manga> {
	let url = format!("{}/series/{}", BASE_API_URL, manga_id);
	let data = Request::new(url, HttpMethod::Get).json()?.as_object()?;

	let mut cover = data.get("thumbnail").as_string()?.read();
	if !cover.contains("https") {
		cover = format!("https://reaperscans.com/_next/image?url=https%3A%2F%2Fmedia.reaperscans.com%2Ffile%2F4SRBHm%2F{}&w=384&q=75", cover)
	}

	let title = data.get("title").as_string()?.read();
	let description = data.get("description").as_string()?.read();
	let id = data.get("series_slug").as_string()?.read();
	let url = format!("{}/series/{}", base_url, id);
	let status = data.get("status").as_string()?.read();

	let manga_status = match status.as_str() {
		"New" => MangaStatus::Unknown,
		"Ongoing" => MangaStatus::Ongoing,
		"Completed" => MangaStatus::Completed,
		"Cancelled" => MangaStatus::Cancelled,
		"Dropped" => MangaStatus::Cancelled,
		"Hiatus" => MangaStatus::Hiatus,
		_ => MangaStatus::Unknown,
	};

	let mut categories: Vec<String> = Vec::new();
	let tags = data.get("tags").as_array()?;
	for tag in tags {
		let tag = tag.as_object()?;
		categories.push(tag.get("name").as_string()?.read());
	}

	Ok(Manga {
		id,
		cover,
		title,
		description,
		url,
		categories,
		status: manga_status,
		viewer: MangaViewer::Scroll,
		..Default::default()
	})
}

pub fn parse_chapter_list(base_url: String, manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/series/{}", BASE_API_URL, manga_id);
	let data = Request::new(url, HttpMethod::Get).json()?.as_object()?;
	let manga_id = data.get("id").as_int()?.to_string();

	let url = format!(
		"{}/chapter/query?page=1&perPage=30&series_id={}",
		BASE_API_URL, manga_id
	);
	let data = Request::new(url, HttpMethod::Get).json()?.as_object()?;
	let mut page = data.get("meta").as_object()?.get("first_page").as_int()?;
	let last_page = data.get("meta").as_object()?.get("last_page").as_int()?;

	let mut all_chapters: Vec<Chapter> = Vec::new();

	while page <= last_page {
		let url = format!(
			"{}/chapter/query?page={}&perPage=30&series_id={}",
			BASE_API_URL, page, manga_id
		);
		let data = Request::new(url, HttpMethod::Get).json()?.as_object()?;

		let chapters = data.get("data").as_array()?;

		for chapter in chapters {
			let chapter = chapter.as_object()?;

			let id = chapter.get("chapter_slug").as_string()?.read();

			let title = if let Ok(title) = chapter.get("chapter_title").as_string() {
				title.read()
			} else {
				String::from("")
			};

			let chapter_number = {
				let chapter_number = chapter
					.get("chapter_name")
					.as_string()
					.unwrap_or_default()
					.read();

				chapter_number
					.replace("Chapter", "")
					.trim()
					.parse::<f32>()
					.unwrap_or(-1.0)
			};

			let url = format!("{}/series/{}/{}", base_url, manga_id, id);

			let date_updated = chapter
				.get("created_at")
				.as_date("yyyy-MM-dd'T'HH:mm:ss.SSSXXX", Some("en_US"), None)
				.unwrap_or(-1.0);

			all_chapters.push(Chapter {
				id,
				title,
				chapter: chapter_number,
				date_updated,
				url,
				..Default::default()
			});
		}
		page += 1;
	}

	Ok(all_chapters)
}

pub fn parse_page_list(
	base_url: String,
	manga_id: String,
	chapter_id: String,
) -> Result<Vec<Page>> {
	let url = format!("{}/series/{}/{}", base_url, manga_id, chapter_id);

	let obj = Request::new(url, HttpMethod::Get).html()?;

	let mut page_list: Vec<Page> = Vec::new();

	for (i, page) in obj.select(".container > .flex > img").array().enumerate() {
		let obj = page.as_node().expect("node array");
		let srcset = obj
			.attr("srcset")
			.read()
			.split(',')
			.next()
			.unwrap_or("")
			.split(' ')
			.next()
			.unwrap_or("")
			.to_string();
		let mut url = format!("{}{}", base_url, srcset);

		if srcset.is_empty() {
			url = obj.attr("src").read();
		}

		page_list.push(Page {
			index: i as i32,
			url,
			..Default::default()
		});
	}

	Ok(page_list)
}

pub fn modify_image_request(base_url: String, request: Request) {
	request.header("Referer", &base_url);
}

fn parse_manga(base_url: &String, json: Request) -> Result<Vec<Manga>> {
	let data = json.json()?.as_object()?.get("data").as_array()?;
	let mut mangas: Vec<Manga> = Vec::new();

	for manga in data {
		let manga = manga.as_object()?;
		let title = manga.get("title").as_string()?.read();
		let mut cover = manga.get("thumbnail").as_string()?.read();

		if !cover.contains("https") {
			cover = format!("https://reaperscans.com/_next/image?url=https%3A%2F%2Fmedia.reaperscans.com%2Ffile%2F4SRBHm%2F{}&w=384&q=75", cover)
		}

		let id = manga.get("series_slug").as_string()?.read();

		let url = format!("{}/series/{}", base_url, id);

		mangas.push(Manga {
			id,
			cover,
			title,
			url,
			..Default::default()
		});
	}

	Ok(mangas)
}
