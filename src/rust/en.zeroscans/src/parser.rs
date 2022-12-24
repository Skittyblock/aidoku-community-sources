use aidoku::{
	error::Result, prelude::format, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Chapter, Filter, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer,
	Page,
};

use crate::helper::*;

// TODO: Zero Scans does not have a search api, they just filter JSON client side
// I tried to implement fuzzy title matching, but rust was being a pain and not
// letting me use the sublime_fuzzy crate, so I gave up
pub fn parse_manga_list(
	base_url: String,
	_filters: Vec<Filter>,
	_page: i32,
) -> Result<MangaPageResult> {
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

	comics.for_each(|manga| {
		let manga = manga.as_object().expect("Failed to get manga as object");

		// let id = manga
		// 	.get("id")
		// 	.as_int()
		// 	.expect("Failed to get manga id as int");
		// let id = format!("{}", id);

		let cover = manga
			.get("cover")
			.as_object()
			.expect("Failed to get manga cover as object")
			.get("horizontal")
			.as_string()
			.expect("Failed to get manga cover as str")
			.read();

		let title = manga
			.get("name")
			.as_string()
			.expect("Failed to get manga name as str")
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

		generes.for_each(|genere| {
			let genere = genere.as_object().expect("Failed to get genere as object");

			let name = genere
				.get("name")
				.as_string()
				.expect("Failed to get genere name as str")
				.read();

			categories.push(name);
		});

		let statuses = manga
			.get("statuses")
			.as_array()
			.expect("Failed to get manga statuses as array");

		let mut manga_status = MangaStatus::Unknown;

		statuses.for_each(|status| {
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
		});

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
	});

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
	todo!()
}

pub fn parse_chapter_list(base_url: String, manga_id: String) -> Result<Vec<Chapter>> {
	todo!()
}

pub fn parse_page_list(
	base_url: String,
	manga_id: String,
	chapter_id: String,
) -> Result<Vec<Page>> {
	todo!()
}

pub fn modify_image_request(base_url: String, request: Request) {
	todo!()
}
