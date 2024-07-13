use aidoku::{
	error::Result, prelude::format, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Chapter, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
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

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				search_query = filter.value.as_string()?.read();
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
			_ => continue,
		}
	}

	if !genres.is_empty() {
		genres.pop();
	}

	let url = format!("{}/query?query_string={}&order=desc&orderBy=total_views&series_type=Comic&page={}&perPage=10&tags_ids=[{}]&adult=true", BASE_API_URL, search_query, page, genres);
	let json = Request::new(url, HttpMethod::Get);
	let manga = parse_manga(&base_url, json)?;
	let has_more = !manga.is_empty();

	Ok(MangaPageResult { manga, has_more })
}

pub fn parse_manga_listing(
	base_url: String,
	listing: Listing,
	page: i32,
) -> Result<MangaPageResult> {
	let list_query = match listing.name.as_str() {
		"Latest Updates" => "latest",
		"Popular" => "total_views",
		"Newest" => "created_at",
		"Alphabetical" => "title",
		_ => "",
	};
	let url = format!("{}/query?query_string=&order=desc&orderBy={}&series_type=Comic&page={}&perPage=10&tags_ids=[]&adult=true", BASE_API_URL, list_query, page);

	let json = Request::new(url, HttpMethod::Get);
	let manga = parse_manga(&base_url, json)?;
	let has_more = !manga.is_empty();

	Ok(MangaPageResult { manga, has_more })
}

pub fn parse_manga_details(base_url: &String, manga_id: String) -> Result<Manga> {
	let url = format!("{}/series/{}", BASE_API_URL, manga_id);
	let data = Request::new(url, HttpMethod::Get).json()?.as_object()?;

	let cover = data.get("thumbnail").as_string()?.read();
	let title = data.get("title").as_string()?.read();
	let description = data.get("description").as_string()?.read();
	let author = data.get("author").as_string()?.read();
	let artist = data.get("studio").as_string()?.read();
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
		author,
		artist,
		description,
		url,
		categories,
		status: manga_status,
		nsfw: MangaContentRating::Nsfw,
		viewer: MangaViewer::Scroll,
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
			let price = chapter.get("price").as_int()?;

			// Only get free chapters
			if price != 0 {
				continue;
			}

			let id = chapter.get("chapter_slug").as_string()?.read();

			let index = id.split('-').collect::<Vec<&str>>();
			let index = String::from(index[1]).parse::<f32>().unwrap_or(-1.0);

			let url = format!("{}/series/{}/{}", base_url, manga_id, id);

			let date_updated = chapter
				.get("created_at")
				.as_date("yyyy-MM-dd'T'HH:mm:ss.SSSXXX", Some("en_US"), None)
				.unwrap_or(-1.0);

			all_chapters.push(Chapter {
				id,
				chapter: index,
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
