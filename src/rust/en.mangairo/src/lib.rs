#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

mod parser;
use parser::{BASE_URL, USER_AGENT};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let url = parser::get_filtered_url(filters, page);
	// aidoku::prelude::println!("get_manga_list: {}", url);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;

	let (manga, has_more) = parser::parse_manga_list(html, page);
	Ok(MangaPageResult { manga, has_more })
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = format!("{}", &manga_id);
	let html = Request::new(url, HttpMethod::Get).html()?;
	parser::parse_manga_details(html, manga_id)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let url = match listing.name.as_str() {
		"Latest" => format!("{BASE_URL}/manga-list/type-latest/ctg-all/state-all/page-{page}"),
		"New Releases" => format!("{BASE_URL}/manga-list/type-newest/ctg-7/state-all/page-{page}"),
		"Hot" => format!("{BASE_URL}/manga-list/type-topview/ctg-all/state-all/page-{page}"),
		"Completed" => {
			format!("{BASE_URL}/manga-list/type-latest/ctg-all/state-completed/page-{page}")
		}
		_ => format!("{BASE_URL}/manga-list/type-latest/ctg-all/state-all/page-{page}"),
	};
	// aidoku::prelude::println!("get_manga_listing: {}", url);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	let (manga, has_more) = parser::parse_manga_list(html, page);

	Ok(MangaPageResult { manga, has_more })
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}", &manga_id);
	// aidoku::prelude::println!("get_chapter_list: {}", url);
	let html = Request::new(url, HttpMethod::Get).html()?;
	parser::get_chapter_list(html)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{}/{}", &manga_id, &chapter_id);
	// aidoku::prelude::println!("get_page_list: {}", url);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	parser::get_page_list(html)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
		.header("Referer", BASE_URL)
		.header("User-Agent", USER_AGENT);
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let parsed_manga_id = parser::parse_incoming_url_manga_id(&url);
	let parsed_chapter_id = parser::parse_incoming_url_chapter_id(&url);
	// aidoku::prelude::println!("handle_url manga id: {:?}", parsed_manga_id);
	// aidoku::prelude::println!("handle_url chapter id: {:?}", parsed_chapter_id);

	if let Some(parsed_manga_id) = parsed_manga_id {
		Ok(DeepLink {
			manga: Some(get_manga_details(parsed_manga_id)?),
			chapter: parsed_chapter_id.map(|chapter_id_value| Chapter {
				id: chapter_id_value,
				..Default::default()
			}),
		})
	} else {
		Err(aidoku::error::AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		})
	}
}
