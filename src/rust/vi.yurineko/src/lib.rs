#![no_std]
pub mod helper;
pub mod parser;
use aidoku::{
	prelude::*, error::Result, std::String, std::Vec, 
	std::net::Request, std::net::HttpMethod, Filter, FilterType, Listing, Manga, 
	MangaPageResult, Chapter, Page, DeepLink,
};
use helper::{get_tag_id, get_search_url, urlencode, listing_map, i32_to_string};
use parser::{parse_manga, parse_chapter};

#[get_manga_list]
pub fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    let mut genre = String::new();
	let mut query = String::new();
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = urlencode(filter.value.as_string()?.read());
			},
			_ => {
				match filter.name.as_str() {
					"Tag" => {
						genre = get_tag_id(filter.value.as_int().unwrap_or(0));
					},
					_ => continue
				}
			}
		}
	}
	let search_url = get_search_url(String::from("https://api.yurineko.net"), query, genre, page);
	let json = Request::new(search_url.as_str(), HttpMethod::Get).json().as_object()?;
	let result = json.get("result").as_array()?;
	let total = json.get("resultCount").as_int().unwrap_or(0);
	let mut manga_arr: Vec<Manga> = Vec::new();
	for manga in result {
		let manga_obj = manga.as_object()?;
		if let Ok(manga) = parse_manga(manga_obj) {
			manga_arr.push(manga);
		}
	}

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: i64::from((page - 1) + 20) < total
	})
}

#[get_manga_listing]
pub fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let mut url = String::from("https://api.yurineko.net/");
	url.push_str(&listing_map(listing.name));
	url.push_str("?page=");
	url.push_str(&i32_to_string(page));

	let result = Request::new(url.as_str(), HttpMethod::Get).json().as_array()?;
	let mut manga_arr: Vec<Manga> = Vec::new();
	for manga in result {
		let manga_obj = manga.as_object()?;
		if let Ok(manga) = parse_manga(manga_obj) {
			manga_arr.push(manga);
		}
	}

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: url.contains("random")
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let mut url = String::from("https://api.yurineko.net/manga/");
	url.push_str(id.as_str());
	let json = Request::new(url.as_str(), HttpMethod::Get).json().as_object()?;
	parse_manga(json)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let mut url = String::from("https://api.yurineko.net/manga/");
	url.push_str(id.as_str());

	let json = Request::new(url.as_str(), HttpMethod::Get).json().as_object()?;
	let chapters = json.get("chapters").as_array()?;
	let scanlators = json.get("team").as_array()?;
	let scanlators_string = scanlators.map(|a| {
		let scanlator_object = a.as_object()?;
		return Ok(scanlator_object.get("name").as_string()?.read())
	})
		.map(|a: Result<String>| a.unwrap_or(String::from("")))
		.collect::<Vec<String>>()
		.join(", ");

	let mut chapter_arr: Vec<Chapter> = Vec::new();
	for chapter in chapters {
		let chapter_obj = chapter.as_object()?;
		if let Ok(chapter) = parse_chapter(String::from(&scanlators_string), chapter_obj) {
			chapter_arr.push(chapter);
		} 
	}
	Ok(chapter_arr)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	let mut url = String::from("https://api.yurineko.net/read/");
	url.push_str(id.as_str());

	let json = Request::new(url.as_str(), HttpMethod::Get).json().as_object()?;
	let pages = json.get("url").as_array()?;
	let mut page_arr: Vec<Page> = Vec::new();
	for (idx, page) in pages.enumerate() {
		page_arr.push(Page { index: idx as i32, url: page.as_string()?.read(), base64: String::new(), text: String::new() })
	}
	Ok(page_arr)
}

#[modify_image_request] 
fn modify_image_request(request: Request) {
    request.header("Referer", "https://yurineko.net").header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.0.4951.54 Safari/537.36 Edg/101.0.1210.39");
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let url = &url[21..]; // remove https://yurineko.net/

	if url.starts_with("manga") { // https://yurineko.net/manga/419
		let id = &url[6..]; // remove manga/
		let manga = get_manga_details(String::from(id))?;
		return Ok(DeepLink {
			manga: Some(manga),
			chapter: None
		})
	} else if url.starts_with("read") { // https://yurineko.net/read/419/5473
		let id = &url[5..]; // remove read/
		let end = match id.find("/") {
			Some(end) => end,
			None => id.len(),
		};
		let manga_id = &id[..end];
		let manga = get_manga_details(String::from(manga_id))?;

		let mut api_url = String::from("https://api.yurineko.net/read/");
		api_url.push_str(id);
		let json = Request::new(api_url.as_str(), HttpMethod::Get).json().as_object()?;
		let chapter_info = json.get("chapterInfo").as_object()?;
		let chapter = parse_chapter(String::from(""), chapter_info)?;
		
		return Ok(DeepLink {
			manga: Some(manga),
			chapter: Some(chapter)
		})

	}
	Err(aidoku::error::AidokuError { reason: aidoku::error::AidokuErrorKind::Unimplemented })
}

