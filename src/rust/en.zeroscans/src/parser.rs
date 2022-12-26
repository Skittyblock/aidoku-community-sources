use aidoku::{
	error::Result, prelude::format, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Chapter, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

use crate::helper::*;

pub fn parse_manga_list(
	base_url: String,
	filters: Vec<Filter>,
	_page: i32,
) -> Result<MangaPageResult> {
	let mut search_query = String::new();

	for filter in filters {
		if filter.kind == FilterType::Title {
			search_query = filter
				.value
				.as_string()
				.expect("Failed to get search filter value")
				.read()
				.to_lowercase();
		}
	}

	let url = format!("{}/swordflake/comics", base_url);

	let json = Request::new(url, HttpMethod::Get)
		.json()
		.expect("Failed to load JSON")
		.as_object()
		.expect("Failed to get JSON as object");

	let data = json
		.get("data")
		.as_object()
		.expect("Failed to get data as object");

	let comics = data
		.get("comics")
		.as_array()
		.expect("Failed to get comics as array");

	let mut mangas: Vec<Manga> = Vec::new();

	for manga in comics {
		let manga = manga.as_object().expect("Failed to get manga as object");

		// let id = manga
		// 	.get("id")
		// 	.as_int()
		// 	.expect("Failed to get manga id as int");
		// let id = format!("{}", id);

		let title = manga
			.get("name")
			.as_string()
			.expect("Failed to get manga name as str")
			.read();

		if !search_query.is_empty() && !title.to_lowercase().contains(&search_query) {
			continue;
		}

		let cover = manga
			.get("cover")
			.as_object()
			.expect("Failed to get manga cover as object")
			.get("horizontal")
			.as_string()
			.expect("Failed to get manga cover as str")
			.read();

		let description = manga
			.get("summary")
			.as_string()
			.expect("Failed to get manga summary as str")
			.read();

		let slug = manga
			.get("slug")
			.as_string()
			.expect("Failed to get manga slug as str")
			.read();

		let url = format!("{}/comics/{}", base_url, slug);

		let generes = manga
			.get("genres")
			.as_array()
			.expect("Failed to get manga genres as array");

		let mut categories: Vec<String> = Vec::new();

		for genere in generes {
			let genere = genere.as_object().expect("Failed to get genere as object");

			let name = genere
				.get("name")
				.as_string()
				.expect("Failed to get genere name as str")
				.read();

			categories.push(name);
		}

		let statuses = manga
			.get("statuses")
			.as_array()
			.expect("Failed to get manga statuses as array");

		let mut manga_status = MangaStatus::Unknown;

		for status in statuses {
			let status = status.as_object().expect("Failed to get status object");

			let name = status
				.get("name")
				.as_string()
				.expect("Failed to get genere name as str")
				.read();

			match name.as_str() {
				"New" => {}
				"Ongoing" => manga_status = MangaStatus::Ongoing,
				"Completed" => manga_status = MangaStatus::Completed,
				"Dropped" => manga_status = MangaStatus::Cancelled,
				"Hiatus" => manga_status = MangaStatus::Hiatus,
				_ => manga_status = MangaStatus::Unknown,
			};
		}

		mangas.push(Manga {
			id: slug,
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description,
			url,
			categories,
			status: manga_status,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll,
		});
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more: false,
	})
}

// The only alternative listing Zero Scans has is new chapters
pub fn parse_manga_listing(
	base_url: String,
	_listing: Listing,
	_page: i32,
) -> Result<MangaPageResult> {
	let url = format!("{}/swordflake/new-chapters", base_url);

	let json = Request::new(url, HttpMethod::Get)
		.json()
		.expect("Failed to load JSON")
		.as_object()
		.expect("Failed to get JSON as object");

	let comics = json
		.get("all")
		.as_array()
		.expect("Failed to get manga as array");

	let mut mangas: Vec<Manga> = Vec::new();

	for manga in comics {
		let manga = manga.as_object().expect("Failed to get manga as object");

		// let id = manga
		// 	.get("id")
		// 	.as_int()
		// 	.expect("Failed to get manga id as int");

		let title = manga
			.get("name")
			.as_string()
			.expect("Failed to get manga title as str")
			.read();

		let slug = manga
			.get("slug")
			.as_string()
			.expect("Failed to get manga slug as str")
			.read();

		let url = format!("{}/comics/{}", base_url, slug);

		let cover = manga
			.get("cover")
			.as_object()
			.expect("Failed to get manga cover as object")
			.get("vertical")
			.as_string()
			.expect("Failed to get manga cover as str")
			.read();

		mangas.push(Manga {
			id: slug,
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
		})
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more: false,
	})
}

pub fn parse_manga_details(base_url: String, manga_id: String) -> Result<Manga> {
	let url = format!("{}/swordflake/comic/{}", base_url, manga_id);

	let json = Request::new(url, HttpMethod::Get)
		.json()
		.expect("Failed to load JSON")
		.as_object()
		.expect("Failed to get JSON as object");

	let data = json
		.get("data")
		.as_object()
		.expect("Failed to get data as object");

	// let id = data
	// 	.get("id")
	// 	.as_int()
	// 	.expect("Failed to get manga id as int");
	// let id = format!("{}", id);

	let cover = data
		.get("cover")
		.as_object()
		.expect("Failed to get manga cover as object")
		.get("full")
		.as_string()
		.expect("Failed to get manga cover as str")
		.read();

	let title = data
		.get("name")
		.as_string()
		.expect("Failed to get manga name as str")
		.read();

	let description = data
		.get("summary")
		.as_string()
		.expect("Failed to get manga summary as str")
		.read();

	let slug = data
		.get("slug")
		.as_string()
		.expect("Failed to get manga slug as str")
		.read();

	let url = format!("{}/comics/{}", base_url, slug);

	let generes = data
		.get("genres")
		.as_array()
		.expect("Failed to get manga genres as array");

	let mut categories: Vec<String> = Vec::new();

	for genere in generes {
		let genere = genere.as_object().expect("Failed to get genere as object");

		let name = genere
			.get("name")
			.as_string()
			.expect("Failed to get genere name as str")
			.read();

		categories.push(name);
	}

	let statuses = data
		.get("statuses")
		.as_array()
		.expect("Failed to get manga statuses as array");

	let mut manga_status = MangaStatus::Unknown;

	for status in statuses {
		let status = status.as_object().expect("Failed to get status as object");

		let name = status
			.get("name")
			.as_string()
			.expect("Failed to get genere name as str")
			.read();

		match name.as_str() {
			"New" => {}
			"Ongoing" => manga_status = MangaStatus::Ongoing,
			"Completed" => manga_status = MangaStatus::Completed,
			"Dropped" => manga_status = MangaStatus::Cancelled,
			"Hiatus" => manga_status = MangaStatus::Hiatus,
			_ => manga_status = MangaStatus::Unknown,
		};
	}

	Ok(Manga {
		id: slug,
		cover,
		title,
		author: String::new(),
		artist: String::new(),
		description,
		url,
		categories,
		status: manga_status,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Scroll,
	})
}

// TODO: Find a way to pass the manga int id to this function instead of making
// another http request just to get the id, currently the manga_id is the slug
// which is used everywhere else BUT here >:(
pub fn parse_chapter_list(base_url: String, manga_id: String) -> Result<Vec<Chapter>> {
	// Get manga id from slug
	let manga_int_id = {
		let url = format!("{}/swordflake/comic/{}", base_url, manga_id);

		let json = Request::new(url, HttpMethod::Get)
			.json()
			.expect("Failed to load JSON")
			.as_object()
			.expect("Failed to get JSON as object");

		let data = json
			.get("data")
			.as_object()
			.expect("Failed to get data as object");

		data.get("id")
			.as_int()
			.expect("Failed to get manga id as int")
	};

	if manga_int_id == 0 {
		return Ok(Vec::new());
	}

	let mut all_chapters: Vec<Chapter> = Vec::new();
	let mut page = 1;

	loop {
		let url = format!(
			"{}/swordflake/comic/{}/chapters?sort=desc&page={}",
			base_url, manga_int_id, &page
		);

		let json = Request::new(url, HttpMethod::Get)
			.json()
			.expect("Failed to load JSON")
			.as_object()
			.expect("Failed to get JSON as object");

		let data = json
			.get("data")
			.as_object()
			.expect("Failed to get data as object");

		let chapters = data
			.get("data")
			.as_array()
			.expect("Failed to get chapters as array");

		let last_page = data
			.get("last_page")
			.as_int()
			.expect("Failed to get last page as int");

		if !chapters.is_empty() {
			for chapter in chapters {
				let chapter = chapter
					.as_object()
					.expect("Failed to get chapter as object");

				let id = chapter
					.get("id")
					.as_int()
					.expect("Failed to get chapter id as int");
				let id = format!("{}", id);

				let chapter_number = chapter
					.get("name")
					.as_float()
					.expect("Failed to get chapter number as float") as f32;

				let date_updated = chapter
					.get("created_at")
					.as_string()
					.expect("Failed to get chapter date as str")
					.read();

				let date_updated = get_date(date_updated);

				let scanlator = chapter.get("group").as_string().unwrap_or("".into()).read();

				let chapter_url = format!("{}/comics/{}/chapters/{}", base_url, manga_id, id);

				all_chapters.push(Chapter {
					id,
					title: String::new(),
					volume: -1.0,
					chapter: chapter_number,
					date_updated,
					scanlator,
					url: chapter_url,
					lang: String::from("en"),
				});
			}
		}

		if page == last_page {
			break;
		} else {
			page += 1;
		}
	}

	Ok(all_chapters)
}

pub fn parse_page_list(
	base_url: String,
	manga_id: String,
	chapter_id: String,
) -> Result<Vec<Page>> {
	let url = format!(
		"{}/swordflake/comic/{}/chapters/{}",
		base_url, manga_id, chapter_id
	);

	let json = Request::new(url, HttpMethod::Get)
		.json()
		.expect("Failed to load JSON")
		.as_object()
		.expect("Failed to get JSON object");

	let data = json
		.get("data")
		.as_object()
		.expect("Failed to get data as object");

	let chapter = data
		.get("chapter")
		.as_object()
		.expect("Failed to get chapter as object");

	let pages = chapter
		.get("high_quality")
		.as_array()
		.expect("Failed to get pages as array");

	let mut page_list: Vec<Page> = Vec::new();

	for (index, page) in pages.enumerate() {
		let page = page.as_string().expect("Failed to get page as str").read();

		page_list.push(Page {
			index: index as i32,
			url: page,
			text: String::new(),
			base64: String::new(),
		});
	}

	Ok(page_list)
}

pub fn modify_image_request(base_url: String, request: Request) {
	request.header("Referer", &base_url);
}
