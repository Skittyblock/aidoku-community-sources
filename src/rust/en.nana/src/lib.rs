#![no_std]
extern crate alloc;

use aidoku::{
	error::Result, prelude::*, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Chapter, Filter, Manga, MangaPageResult, Page,
};
use alloc::string::ToString;

mod parser;

const BASE_URL: &str = "https://nana.my.id";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let url = parser::get_filtered_url(filters, page);

	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	let html_next_page = Request::new(url.as_str(), HttpMethod::Get).html()?;

	let result = parser::parse_search(html);

	Ok(MangaPageResult {
		manga: result,
		has_more: !html_next_page
			.select("a.paginate_button.current + a.paginate_button")
			.array()
			.is_empty(),
	})
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn get_manga_details(manga_rid: i32) -> i32 {
	let manga = aidoku::std::ValueRef::new(manga_rid).as_object().unwrap();
	let url = format!(
		"{}/reader/{}",
		BASE_URL,
		manga.get("id").as_string().unwrap().read()
	);

	let (title, cover) =
		if manga.get("title").as_string().is_err() && manga.get("cover").as_string().is_err() {
			let html = match Request::new(url.as_str(), HttpMethod::Get).html() {
				Ok(req) => req,
				Err(_) => return -1,
			};

			let title = html
				.select("#archivePagesOverlay .spanh3reader")
				.text()
				.read()
				.trim()
				.to_string();

			let img = html.select("a#display img").attr("src").read();
			let cover = if img.starts_with('/') {
				format!("{}{}", BASE_URL, img)
			} else {
				img
			}
			.replace("/image/page", "/image/thumbnails");

			(title, cover)
		} else {
			(
				manga.get("title").as_string().unwrap().read(),
				manga.get("cover").as_string().unwrap().read(),
			)
		};

	let mut categories: Vec<String> = Vec::new();
	if let Ok(tags) = manga.get("tags").as_array() {
		for tag in tags {
			let tag = match tag.as_string() {
				Ok(node) => node.read(),
				Err(_) => return -1,
			};
			categories.push(tag);
		}
	}

	let author = match manga.get("author").as_string() {
		Ok(author) => author.read(),
		Err(_) => String::new(),
	};

	let url = match manga.get("url").as_string() {
		Ok(url) => url.read(),
		Err(_) => url,
	};

	Manga {
		id: manga.get("id").as_string().unwrap().read(),
		cover,
		title,
		author,
		artist: String::new(),
		description: String::new(),
		url,
		categories,
		status: aidoku::MangaStatus::Completed,
		nsfw: aidoku::MangaContentRating::Nsfw,
		viewer: aidoku::MangaViewer::Scroll,
	}
	.create()
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/reader/{}", BASE_URL, id);

	Ok(Vec::from([Chapter {
		id,
		title: String::from("Chapter 1"),
		volume: -1.0,
		chapter: 1.0,
		url,
		date_updated: 0.0,
		scanlator: String::new(),
		lang: String::from("en"),
	}]))
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!(
		"{}/api/archives/{}/extractthumbnails",
		BASE_URL, &chapter_id
	);

	let request = Request::new(url, HttpMethod::Get).header("User-Agent", "Aidoku");
	let json = request.json()?.as_object()?;

	parser::get_page_list(json)
}

#[modify_image_request]
pub fn modify_image_request(request: Request) {
	request.header("Referer", BASE_URL);
}
