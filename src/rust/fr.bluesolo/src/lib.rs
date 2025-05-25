#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::net::HttpMethod,
	std::net::Request,
	std::{String, Vec},
	Chapter, Filter, FilterType, Manga, MangaPageResult, Page, DeepLink, MangaStatus, MangaViewer,
	MangaContentRating,
};

const BASE_URL: &str = "https://bluesolo.org";
const API_BASE_URL: &str = "https://bluesolo.org/api";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, _page: i32) -> Result<MangaPageResult> {
	let mut search_query = String::new();
	
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				search_query = urlencode(&filter.value.as_string()?.read());
			}
			_ => continue,
		}
	}
	
	let url = if !search_query.is_empty() {
		format!("{}/search/{}", API_BASE_URL, search_query)
	} else {
		format!("{}/comics", API_BASE_URL)
	};
	
	let json = Request::new(&url, HttpMethod::Get).json()?;

	let comics = json.as_object()?
		.get("comics")
		.as_array()
		.unwrap_or_default();

	let mut manga = Vec::new();
	for comic in comics {
		let obj = comic.as_object()?;
		if let Ok(id) = obj.get("slug").as_string().map(|s| s.read()) {
			if let (Ok(title), Ok(cover)) = (
				obj.get("title").as_string().map(|s| s.read()),
				obj.get("thumbnail").as_string().map(|s| s.read())
			) {
				let nsfw = obj.get("adult")
					.as_int()
					.map(|a| if a == 1 { MangaContentRating::Nsfw } else { MangaContentRating::Safe })
					.unwrap_or(MangaContentRating::Safe);

				manga.push(Manga {
					id,
					title,
					cover,
					author: String::new(),
					artist: String::new(),
					description: String::new(),
					url: String::new(),
					categories: Vec::new(),
					status: MangaStatus::Unknown,
					nsfw,
					viewer: MangaViewer::Rtl,
				});
			}
		}
	}

	Ok(MangaPageResult {
		manga,
		has_more: false,
	})
}

fn urlencode(string: &str) -> String {
	let mut result = String::with_capacity(string.len() * 3);
	for b in string.bytes() {
		match b {
			// Alphanumeric characters and - . _ ~ stay the same
			b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => result.push(b as char),
			// Space becomes %20
			b' ' => {
				result.push('%');
				result.push('2');
				result.push('0');
			},
			// Everything else gets percent encoded
			_ => {
				result.push('%');
				result.push(hex(b >> 4));
				result.push(hex(b & 15));
			}
		}
	}
	result
}

fn hex(n: u8) -> char {
	if n < 10 {
		(n + b'0') as char
	} else {
		(n - 10 + b'A') as char
	}
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{}/comics/{}", API_BASE_URL, id);
	let json = Request::new(&url, HttpMethod::Get).json()?;

	let root = json.as_object()?;
	let comic = root.get("comic").as_object()?;
	
	let slug = comic.get("slug").as_string()?.read();
	let title = comic.get("title").as_string()?.read();
	let cover = comic.get("thumbnail").as_string()?.read();
	
	let categories = comic.get("genres")
		.as_array()
		.map(|arr| {
			let mut genres = Vec::new();
			for genre in arr {
				if let Ok(genre_obj) = genre.as_object() {
					if let Ok(name) = genre_obj.get("name").as_string() {
						genres.push(name.read());
					}
				}
			}
			genres
		})
		.unwrap_or_default();
	
	let nsfw = comic.get("adult")
		.as_int()
		.map(|a| if a == 1 { MangaContentRating::Nsfw } else { MangaContentRating::Safe })
		.unwrap_or(MangaContentRating::Safe);
	
	Ok(Manga {
		id: slug.clone(),
		title,
		cover,
		author: comic.get("author").as_string().map(|s| s.read()).unwrap_or_default(),
		artist: comic.get("artist").as_string().map(|s| s.read()).unwrap_or_default(),
		url: format!("{}/comics/{}", BASE_URL, slug),
		categories,
		description: comic.get("description").as_string().map(|s| s.read()).unwrap_or_default(),
		status: comic.get("status")
			.as_string()
			.map(|s| match s.read().as_str() {
				"En cours" => MangaStatus::Ongoing,
				"Terminé" => MangaStatus::Completed,
				"En hiatus" => MangaStatus::Hiatus,
				"Annulé" => MangaStatus::Cancelled,
				_ => MangaStatus::Unknown,
			})
			.unwrap_or(MangaStatus::Unknown),
		nsfw,
		viewer: MangaViewer::Rtl,
	})
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/comics/{}", API_BASE_URL, manga_id);
	let json = Request::new(&url, HttpMethod::Get).json()?;

	let root = json.as_object()?;
	let comic = root.get("comic").as_object()?;
	let chapters = comic.get("chapters").as_array()?;
	
	let mut chapter_list = Vec::new();

	for chapter in chapters {
		let obj = chapter.as_object()?;
		
		let chapter_num = obj.get("chapter").as_float().unwrap_or_default() as f32;
		let subchapter = match obj.get("subchapter").as_float() {
			Ok(num) => Some(num as f32),
			_ => None
		};
		
		let id = obj.get("slug_lang_vol_ch_sub").as_string()?.read();
		let title = obj.get("title")
			.as_string()
			.map(|s| s.read())
			.unwrap_or_else(|_| String::new());
		
		let volume = obj.get("volume")
			.as_float()
			.map(|v| v as f32)
			.unwrap_or(-1.0);
			
		let scanlator = obj.get("teams")
			.as_array()
			.map(|teams| {
				let mut names = Vec::new();
				for team in teams {
					if let Ok(team_obj) = team.as_object() {
						if let Ok(name) = team_obj.get("name").as_string() {
							names.push(name.read());
						}
					}
				}
				names.join(", ")
			})
			.unwrap_or_default();

		let url = if let Some(sub) = subchapter {
			format!("{}/read/{}/fr/ch/{}/sub/{}", BASE_URL, manga_id, chapter_num as i32, sub as i32)
		} else {
			format!("{}/read/{}/fr/ch/{}", BASE_URL, manga_id, chapter_num as i32)
		};

		let chapter_num = if let Some(sub) = subchapter {
			chapter_num + (sub / 10.0)
		} else {
			chapter_num
		};

		chapter_list.push(Chapter {
			id: id.clone(),
			title,
			volume,
			chapter: chapter_num,
			date_updated: obj.get("published_on")
				.as_date("yyyy-MM-dd'T'HH:mm:ss.SSSSSSZ", Some("fr"), None)
				.unwrap_or_default(),
			scanlator,
			url,
			lang: String::from("fr"),
		});
	}

	Ok(chapter_list)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let parts: Vec<&str> = chapter_id.split('-').collect();
	if parts.len() != 4 {
		return Ok(Vec::new());
	}
	
	let chapter_num = parts[2];
	let sub_part = parts[3];
	
	let url = if sub_part == "N" {
		format!("{}/read/{}/fr/ch/{}", API_BASE_URL, manga_id, chapter_num)
	} else {
		format!("{}/read/{}/fr/ch/{}/sub/{}", API_BASE_URL, manga_id, chapter_num, sub_part)
	};

	let json = Request::new(&url, HttpMethod::Get).json()?;
	let root = json.as_object()?;
	let chapter = root.get("chapter").as_object()?;
	let pages = chapter.get("pages").as_array()?;
	
	let mut page_list = Vec::new();

	for (index, url_value) in pages.enumerate() {
		if let Ok(url) = url_value.as_string().map(|s| s.read()) {
			page_list.push(Page {
				index: index as i32,
				url,
				base64: String::new(),
				text: String::new(),
			});
		}
	}

	Ok(page_list)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.header("Referer", BASE_URL);
}

#[handle_url]
pub fn handle_url(_url: String) -> Result<DeepLink> {
	todo!()
}
