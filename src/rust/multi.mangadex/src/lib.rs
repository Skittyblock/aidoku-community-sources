#![no_std]
extern crate alloc;
use aidoku::{
	error::Result, prelude::*, std::defaults::defaults_get, std::net::HttpMethod,
	std::net::Request, std::ObjectRef, std::String, std::Vec, Chapter, DeepLink, Filter,
	FilterType, Listing, Manga, MangaPageResult, Page,
};
use alloc::string::ToString;
mod parser;

#[link(wasm_import_module = "net")]
extern "C" {
	fn set_rate_limit(rate_limit: i32);
	fn set_rate_limit_period(period: i32);
}

fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if curr.is_ascii_alphanumeric() {
			result.push(curr);
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}

	String::from_utf8(result).unwrap_or_default()
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn initialize() {
	set_rate_limit(3);
	set_rate_limit_period(1);
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let offset = (page - 1) * 20;
	let mut url =
		String::from("https://api.mangadex.org/manga/?includes[]=cover_art&limit=20&offset=");
	url.push_str(&offset.to_string());

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				url.push_str("&title=");
				url.push_str(&urlencode(filter.value.as_string()?.read()));
			}
			FilterType::Author => {
				url.push_str("&author=");
				url.push_str(&urlencode(filter.value.as_string()?.read()));
			}
			FilterType::Check => {
				let value = filter.value.as_int().unwrap_or(-1);
				if value < 0 {
					continue;
				}
				if let Ok(id) = filter.object.get("id").as_string() {
					let mut id = id.read();
					if value == 0 {
						id = id.replace("&originalLanguage", "&excludedOriginalLanguage");
					}
					url.push_str(&id);
				}
			}
			FilterType::Genre => {
				// Run `python scripts/update_tags.py` to fetch tags from https://api.mangadex.org/manga/tag
				let tag = filter.object.get("id").as_string()?.read();
				match filter.value.as_int().unwrap_or(-1) {
					0 => url.push_str("&excludedTags[]="),
					1 => url.push_str("&includedTags[]="),
					_ => continue,
				}
				url.push_str(&tag);
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int().unwrap_or(0);
				let ascending = value.get("ascending").as_bool().unwrap_or(false);
				url.push_str("&order[");
				url.push_str(match index {
					0 => "latestUploadedChapter",
					1 => "relevance",
					2 => "followedCount",
					3 => "createdAt",
					4 => "updatedAt",
					5 => "title",
					_ => continue,
				});
				url.push_str("]=");
				url.push_str(if ascending { "asc" } else { "desc" });
			}
			_ => continue,
		}
	}

	let json = Request::new(&url, HttpMethod::Get).json().as_object()?;

	let data = json.get("data").as_array()?;

	let mut manga_arr: Vec<Manga> = Vec::new();

	for manga in data {
		let manga_obj = manga.as_object()?;
		if let Ok(manga) = parser::parse_basic_manga(manga_obj) {
			manga_arr.push(manga);
		}
	}

	let total = json.get("total").as_int().unwrap_or(0) as i32;

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: offset + 20 < total,
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let mut filters: Vec<Filter> = Vec::new();

	let mut selection = ObjectRef::new();

	if listing.name == "Popular" {
		selection.set("index", 2i32.into());
		selection.set("ascending", false.into());
		filters.push(Filter {
			kind: FilterType::Sort,
			name: String::from("Sort"),
			value: selection.0.clone(),
			object: selection,
		});
	} else if listing.name == "Latest" {
		// get recently published chapters
		let offset = (page - 1) * 20;
		let mut url = String::from("https://api.mangadex.org/chapter?includes[]=manga&order[publishAt]=desc&includeFutureUpdates=0&limit=20&offset=");
		url.push_str(&offset.to_string());
		if let Ok(languages) = defaults_get("languages").as_array() {
			for lang in languages {
				if let Ok(lang) = lang.as_string() {
					url.push_str("&translatedLanguage[]=");
					url.push_str(&lang.read());
				}
			}
		}

		let json = Request::new(&url, HttpMethod::Get).json().as_object()?;

		let data = json.get("data").as_array()?;

		let mut manga_arr: Vec<Manga> = Vec::new();
		let mut manga_ids: Vec<String> = Vec::new();

		for chapter in data {
			if let Ok(chapter_obj) = chapter.as_object() {
				if let Ok(relationships) = chapter_obj.get("relationships").as_array() {
					for relationship in relationships {
						if let Ok(relationship_obj) = relationship.as_object() {
							let relation_type = relationship_obj.get("type").as_string()?.read();
							if relation_type == "manga" {
								let id = relationship_obj.get("id").as_string()?.read();
								if manga_ids.contains(&id) {
									continue;
								}
								if let Ok(parsed_manga) = get_manga_details(id) {
									manga_ids.push(parsed_manga.id.clone());
									manga_arr.push(parsed_manga);
								}
								break;
							}
						}
					}
				}
			}
		}

		let total = json.get("total").as_int().unwrap_or(0) as i32;

		return Ok(MangaPageResult {
			manga: manga_arr,
			has_more: offset + 20 < total,
		});
	}

	get_manga_list(filters, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let mut url = String::from("https://api.mangadex.org/manga/");
	url.push_str(&id);
	url.push_str("?includes[]=cover_art&includes[]=author&includes[]=artist");
	let json = Request::new(&url, HttpMethod::Get).json().as_object()?;

	let data = json.get("data").as_object()?;

	parser::parse_full_manga(data)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let mut url = String::from("https://api.mangadex.org/manga/");
	url.push_str(&id);
	url.push_str("/feed?order[volume]=desc&order[chapter]=desc&limit=500&contentRating[]=pornographic&contentRating[]=erotica&contentRating[]=suggestive&contentRating[]=safe&includes[]=scanlation_group");
	if let Ok(languages) = defaults_get("languages").as_array() {
		for lang in languages {
			if let Ok(lang) = lang.as_string() {
				url.push_str("&translatedLanguage[]=");
				url.push_str(&lang.read());
			}
		}
	}
	if let Ok(groups_string) = defaults_get("blockedGroups").as_string() {
		groups_string.read().split(',').for_each(|group| {
			url.push_str("&excludedGroups[]=");
			url.push_str(group.trim());
		});
	}
	let mut offset = 0;
	let mut chapters: Vec<Chapter> = Vec::new();
	loop {
		let mut url_offset = url.clone();
		url_offset.push_str("&offset=");
		url_offset.push_str(&offset.to_string());
		let json = Request::new(&url_offset, HttpMethod::Get)
			.json()
			.as_object()?;

		let data = json.get("data").as_array()?;
		for chapter in data {
			if let Ok(chapter_obj) = chapter.as_object() {
				if let Ok(chapter) = parser::parse_chapter(chapter_obj) {
					chapters.push(chapter);
				}
			}
		}

		let total = json.get("total").as_int().unwrap_or(0);
		if offset + 500 >= total {
			break;
		}
		offset += 500;
	}

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	let mut url = String::from("https://api.mangadex.org/at-home/server/");
	url.push_str(&id);
	let json = Request::new(&url, HttpMethod::Get).json().as_object()?;

	let chapter = json.get("chapter").as_object()?;
	let data = chapter
		.get(if defaults_get("dataSaver").as_bool().unwrap_or(false) {
			"dataSaver"
		} else {
			"data"
		})
		.as_array()?;

	let base_url = json.get("baseUrl").as_string()?.read();
	let hash = chapter.get("hash").as_string()?.read();

	let mut pages: Vec<Page> = Vec::new();

	for (i, page) in data.enumerate() {
		let data_string = page.as_string()?.read();
		let mut url = String::new();
		url.push_str(&base_url);
		if defaults_get("dataSaver").as_bool().unwrap_or(false) {
			url.push_str("/data-saver/");
		} else {
			url.push_str("/data/");
		}
		url.push_str(&hash);
		url.push('/');
		url.push_str(&data_string);

		pages.push(Page {
			index: i as i32,
			url,
			base64: String::new(),
			text: String::new(),
		});
	}

	Ok(pages)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let url = &url[21..]; // remove "https://mangadex.org/"

	if url.starts_with("title") {
		// ex: https://mangadex.org/title/a96676e5-8ae2-425e-b549-7f15dd34a6d8/komi-san-wa-komyushou-desu
		let id = &url[6..]; // remove "title/"
		let end = match id.find('/') {
			Some(end) => end,
			None => id.len(),
		};
		let manga_id = &id[..end];
		let manga = get_manga_details(String::from(manga_id))?;

		return Ok(DeepLink {
			manga: Some(manga),
			chapter: None,
		});
	} else if url.starts_with("chapter") {
		// ex: https://mangadex.org/chapter/56eecc6f-1a4e-464c-b6a4-a1cbdfdfd726/1
		let id = &url[8..]; // remove "chapter/"
		let end = match id.find('/') {
			Some(end) => end,
			None => id.len(),
		};
		let chapter_id = &id[..end];

		let mut url = String::from("https://api.mangadex.org/chapter/");
		url.push_str(chapter_id);

		let json = Request::new(&url, HttpMethod::Get).json().as_object()?;

		let chapter_obj = json.get("data").as_object()?;
		let relationships = chapter_obj.get("relationships").as_array()?;
		for relationship in relationships {
			if let Ok(relationship_obj) = relationship.as_object() {
				let relation_type = relationship_obj.get("type").as_string()?.read();
				if relation_type == "manga" {
					let manga_id = relationship_obj.get("id").as_string()?.read();
					let manga = get_manga_details(manga_id)?;
					let chapter = parser::parse_chapter(chapter_obj)?;
					return Ok(DeepLink {
						manga: Some(manga),
						chapter: Some(chapter),
					});
				}
			}
		}
	}

	Err(aidoku::error::AidokuError {
		reason: aidoku::error::AidokuErrorKind::Unimplemented,
	})
}
