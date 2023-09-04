#![no_std]
use aidoku::{
	error::{AidokuError, Result},
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
	let mut result: Vec<Manga> = Vec::new();

	let mut url = String::new();
	parser::get_filtered_url(filters, page, &mut url);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;

	parser::parse_manga_list(html, &mut result);

	if result.len() >= 50 {
		Ok(MangaPageResult {
			manga: result,
			has_more: true,
		})
	} else {
		Ok(MangaPageResult {
			manga: result,
			has_more: false,
		})
	}
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = format!("https://w.mangairo.com/{}", &manga_id);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	parser::parse_manga(html, manga_id)
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
		_ => String::from(BASE_URL),
	};
	parser::parse_manga_listing(url)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("https://w.mangairo.com/{}", &manga_id);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	parser::get_chapter_list(html)
}

#[get_page_list]
fn get_page_list(chapter_id: String, manga_id: String) -> Result<Vec<Page>> {
	let url = format!("https://w.mangairo.com/{}/{}", &manga_id, &chapter_id);
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
	let parsed_manga_id = parser::parse_incoming_url_manga_id(url.clone());
	let parsed_chapter_id = parser::parse_incoming_url_chapter_id(url);

	if parsed_manga_id.is_none() {
		return Err(aidoku::error::AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		})
	}
	Ok(DeepLink {
		manga: Some(get_manga_details(parsed_manga_id.unwrap())?),
		chapter: if parsed_chapter_id.is_some() {
			Some(Chapter {
				id: parsed_chapter_id.unwrap(),
				..Default::default()
			})
		} else {
			None
		},
	})
}
