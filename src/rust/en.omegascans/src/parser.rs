use aidoku::{
	error::Result, helpers::uri::encode_uri_component, prelude::format, std::net::HttpMethod,
	std::net::Request, std::String, std::Vec, Chapter, Filter, FilterType, Manga,
	MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page, Listing
};

use crate::BASE_API_URL;

pub fn parse_manga_list(base_url: String, filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut search_query = String::new();
	let mut status = String::new();
	let mut genres = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					search_query.push_str(&encode_uri_component(filter_value.read()));
				}
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
			FilterType::Select => match filter.name.as_str() {
				"Status" => {
					status = match filter.value.as_int().unwrap_or(-1) {
						0 => String::from("All"),
						1 => String::from("Ongoing"),
						2 => String::from("Hiatus"),
						3 => String::from("Dropped"),
						4 => String::from("Completed"),
						5 => String::from("Cancelled"),
						_ => continue,
					};
				}
				_ => continue,
			},
			_ => continue,
		}
	}

	if !genres.is_empty() {
		genres.pop();
	}

	let url = format!("{}/query?query_string={}&series_status={}&order=desc&orderBy=total_views&series_type=Comic&page={}&perPage=10&tags_ids=[{}]", BASE_API_URL, search_query, status, page, genres);
	let json = Request::new(&url, HttpMethod::Get);
	let manga = parse_manga(&base_url, json)?;
	let has_more = is_last_page(&url);

	Ok(MangaPageResult {
		manga,
		has_more,
	})
}

pub fn parse_manga_listing(base_url: String, listing: Listing, page: i32) -> Result<MangaPageResult> {
	let list_query = match listing.name.as_str() {
		"Latest" => "latest",
		"Popular" => "total_views",
		"Alphabetical" => "title",
		_ => "",
	};
	let url = format!("{}/query?query_string=&series_status=All&order=desc&orderBy={}&series_type=Comic&page={}&perPage=10&tags_ids=[]", BASE_API_URL, list_query, page);

	let json = Request::new(&url, HttpMethod::Get);
	let manga = parse_manga(&base_url, json)?;
	let has_more = is_last_page(&url);

	Ok(MangaPageResult {
		manga,
		has_more,
	})
}

pub fn parse_manga_details(base_url: &String, manga_id: String) -> Result<Manga> {
	let url = format!("{}/series/{}", BASE_API_URL, manga_id);

	let data = Request::new(url, HttpMethod::Get).json()?.as_object()?;
	let cover = data.get("thumbnail").as_string()?.read();
	let title = data.get("title").as_string()?.read();
	let description = data.get("description").as_string()?.read();
	let author = data.get("author").as_string()?.read();
	let artist = data.get("studio").as_string()?.read();

	let id = manga_id;
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

	Ok(Manga {
		id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		status: manga_status,
		nsfw: MangaContentRating::Nsfw,
		viewer: MangaViewer::Scroll,
		..Default::default()
	})
}

pub fn parse_chapter_list(base_url: String, manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/series/{}", BASE_API_URL, manga_id);
	let data = Request::new(url, HttpMethod::Get).json()?.as_object()?;

	let mut all_chapters: Vec<Chapter> = Vec::new();

	let seasons = data.get("seasons").as_array()?;

	for season in seasons {
		let season = season.as_object()?;
		let chapters = season.get("chapters").as_array()?;

		for chapter in chapters {
			let chapter = chapter.as_object()?;
			let price = chapter.get("price").as_int()?;

			// Only get free chapters
			if price != 0 {
				continue;
			}

			let index = chapter.get("index").as_string()?.read();
			let id = chapter.get("chapter_slug").as_string()?.read();
			let url = format!("{}/series/{}/{}", base_url, manga_id, id);

			let date_updated = chapter
				.get("created_at")
				.as_date("yyyy-MM-dd'T'HH:mm:ss.SSSXXX", Some("en_US"), None)
				.unwrap_or(-1.0);

			all_chapters.push(Chapter {
				id,
				chapter: index.parse::<f32>().unwrap_or_default(),
				date_updated,
				url,
				..Default::default()
			});
		}
	}

	Ok(all_chapters)
}

pub fn parse_page_list(base_url: String, manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{}/series/{}/{}", base_url, manga_id, chapter_id);

	let obj = Request::new(url, HttpMethod::Get).html()?;

	let mut page_list: Vec<Page> = Vec::new();

	for (i, page) in obj.select("img").array().enumerate() {
		let obj = page.as_node().expect("node array");
		let mut url = obj.attr("data-src").read();

		if url.is_empty() {
			url = obj.attr("src").read();
		}

		page_list.push(Page {
			index: i as i32,
			url,
			..Default::default()
		});
	}

	// Remove icon.png and banners from top and bottom
	page_list.remove(0);
	page_list.remove(0);
	page_list.pop();
	page_list.pop();

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
		let cover = manga.get("thumbnail").as_string()?.read();
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

fn is_last_page(url: &String) -> bool {
	let json = Request::new(url, HttpMethod::Get);
	!json.json().expect("").as_object().expect("").get("data").as_array().expect("").is_empty()
}