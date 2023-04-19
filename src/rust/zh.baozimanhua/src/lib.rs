#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, Manga, MangaPageResult, Page,
};

mod parser;

const BASE_URL: &str = "https://www.baozimh.com";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut url = String::new();
	parser::get_filtered_url(filters, page, &mut url);

	let request = parser::request_get(&mut url);
	if url.contains("/api/bzmhq/amp_comic_list") {
		return parser::parse_home_page(request.json()?);
	}
	parser::parse_search_page(request.html()?)
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let mut url = format!("{}/comic/{}", BASE_URL, manga_id);
	parser::get_manga_details(parser::request_get(&mut url).html()?, manga_id)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let mut url = format!("{}/comic/{}", BASE_URL, manga_id);
	parser::get_chapter_list(parser::request_get(&mut url).html()?, manga_id)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut url = format!(
		"{}/comic/chapter/{}/0_{}.html",
		BASE_URL, manga_id, chapter_id
	);
	parser::get_page_list(parser::request_get(&mut url).html()?)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.header("Referer", BASE_URL).header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_3_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36");
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let mut deep_link = url;
	let (manga_id, chapter_id) = parser::parse_deep_link(&mut deep_link);

	let manga = manga_id
		.is_some()
		.then(|| get_manga_details(manga_id.expect("manga_id String")).expect("Manga"));
	let chapter = chapter_id.is_some().then(|| Chapter {
		id: chapter_id.expect("chapter_id String"),
		..Default::default()
	});

	Ok(DeepLink { manga, chapter })
}
