#![no_std]
#![allow(clippy::mut_range_bound)]
#![feature(try_blocks)]

use aidoku::{
	error::Result,
	prelude::*,
	std::{
		defaults::defaults_get,
		html::Node,
		json::parse,
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, Page,
};

pub mod helper;
mod parser;

mod model;
use model::{Nepnep, Pattern, Size, SortOptions};

pub fn init_cache(cache: &mut Nepnep) {
	if let Ok(url_str) = defaults_get("sourceURL")
		.expect("missing sourceURL")
		.as_string()
	{
		let mut url = url_str.read();
		url.push_str(cache.path());

		let html = match Request::new(&url, HttpMethod::Get).html() {
			Ok(html) => html,
			Err(_) => return,
		};

		let result = html.outer_html().read();
		let final_str = helper::string_between(&result, cache.start(), cache.end(), 1);

		match cache {
			Nepnep::Directory { items } => match serde_json::from_str(final_str.as_str()) {
				Ok(dir) => *items = dir,
				Err(err) => {
					println!("Unable to serialize :{:?}", err);
				}
			},
			Nepnep::HotUpdate { items } => match serde_json::from_str(final_str.as_str()) {
				Ok(dir) => *items = dir,
				Err(err) => {
					println!("Unable to serialize :{:?}", err);
				}
			},
		}
	}
}

static mut CACHED_MANGA_ID: Option<String> = None;
static mut CACHED_MANGA: Option<String> = None;

static mut CACHED_DIR: Nepnep = Nepnep::Directory { items: Vec::new() };
static mut CACHED_HOT_UPDATES: Nepnep = Nepnep::HotUpdate { items: Vec::new() };

// Cache manga page html
pub fn cache_manga_page(id: &str) {
	if unsafe { CACHED_MANGA.is_some() } && unsafe { CACHED_MANGA_ID.clone().unwrap() } == id {
		return;
	}
	if let Ok(url_str) = defaults_get("sourceURL")
		.expect("missing sourceURL")
		.as_string()
	{
		let mut url = url_str.read();
		url.push_str("/manga/");
		url.push_str(id);
		unsafe {
			CACHED_MANGA = Request::new(&url, HttpMethod::Get).string().ok();
			CACHED_MANGA_ID = Some(String::from(id));
		};
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	if unsafe { &CACHED_DIR }.len() == 0 {
		init_cache(unsafe { &mut CACHED_DIR })
	}

	let mut dir = match unsafe { CACHED_DIR.clone() } {
		Nepnep::Directory { items } => items,
		_ => panic!("Unexpected type"),
	};

	let offset = (page as usize - 1) * 20;

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				let title = filter.value.as_string()?.read().to_lowercase();

				let mut i = 0;
				let mut size = dir.len();
				for _ in 0..size {
					if i >= size || i >= offset + 20 {
						break;
					}
					let manga = match dir.get(i) {
						Some(manga) => manga,
						None => {
							i += 1;
							continue;
						}
					};

					// check both series name and alt titles
					if manga.title.to_lowercase().contains(&title)
						|| manga
							.alt_titles
							.iter()
							.any(|x| x.to_lowercase().contains(&title))
					{
						i += 1;
						continue;
					}
					// no match, remove
					dir.remove(i);
					size -= 1;
				}
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};

				let idx = value.get("index").as_int().unwrap_or(0) as i32;
				let opt = SortOptions::from(idx);

				match opt {
					// Site by default sorts to A-Z
					SortOptions::AZ => continue,
					SortOptions::ZA => dir.sort_by(|a, b| b.title.cmp(&a.title)),
					SortOptions::RecentlyReleasedChapter => {
						dir.sort_by(|a, b| b.last_updated.cmp(&a.last_updated))
					}
					SortOptions::YearReleasedNewest => dir.sort_by(|a, b| b.year.cmp(&a.year)),
					SortOptions::YearReleasedOldest => dir.sort_by(|a, b| a.year.cmp(&b.year)),
					SortOptions::MostPopularAllTime => dir.sort_by(|a, b| b.views.cmp(&a.views)),
					SortOptions::MostPopularMonthly => {
						dir.sort_by(|a, b| b.views_month.cmp(&a.views_month))
					}
					SortOptions::LeastPopular => dir.sort_by(|a, b| a.views.cmp(&b.views)),
				}
			}
			_ => continue,
		}
	}

	let end = if dir.len() > offset + 20 {
		offset + 20
	} else {
		dir.len()
	};

	let mut manga: Vec<Manga> = Vec::with_capacity(20);

	for i in offset..end {
		let manga_obj = dir.get(i);
		match manga_obj {
			Some(obj) => manga.push(parser::parse_basic_manga(obj)?),
			None => panic!("Couldn't find index: {}", i),
		}
	}

	Ok(MangaPageResult {
		manga,
		has_more: dir.len() > end,
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let listing_name = listing.name.as_str();
	match listing_name {
		"Hot Updates" => {
			if unsafe { &CACHED_HOT_UPDATES }.len() == 0 {
				init_cache(unsafe { &mut CACHED_HOT_UPDATES })
			}
		}
		_ => {
			panic!("Received unexpected listing: {}", listing_name);
		}
	}

	let dir = match unsafe { CACHED_HOT_UPDATES.clone() } {
		Nepnep::HotUpdate { items } => items,
		_ => panic!("Unexpected type"),
	};

	let offset = (page as usize - 1) * 20;

	let end = if dir.len() > offset + 20 {
		offset + 20
	} else {
		dir.len()
	};

	let mut manga: Vec<Manga> = Vec::with_capacity(20);
	for i in offset..end {
		let manga_obj = dir.get(i);
		match manga_obj {
			Some(obj) => manga.push(parser::parse_manga_listing(obj)?),
			None => panic!("Couldn't find index: {}", i),
		}
	}
	Ok(MangaPageResult {
		manga,
		has_more: dir.len() > end,
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	cache_manga_page(&id);
	let html = unsafe { Node::new(CACHED_MANGA.clone().unwrap().as_bytes()) }?;

	let mut url = defaults_get("sourceURL")?.as_string()?.read();
	url.push_str("/manga/");
	url.push_str(&id);

	parser::parse_full_manga(id, url, html)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	cache_manga_page(&id);
	let result = unsafe { CACHED_MANGA.clone().unwrap() };

	let start_loc = result.find("vm.Chapters = ").unwrap_or(0) + 14;
	let half_json = &result[start_loc..];
	let json_end = half_json.find("];").unwrap_or(half_json.len() - 1) + 1;
	let json = &half_json[..json_end];

	let chapter_arr = parse(json.as_bytes())?.as_array()?;

	let mut chapters: Vec<Chapter> = Vec::with_capacity(chapter_arr.len());

	for chapter in chapter_arr {
		let chapter_obj = chapter.as_object()?;
		chapters.push(parser::parse_chapter(&id, chapter_obj)?);
	}

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut url = defaults_get("sourceURL")?.as_string()?.read();
	url.push_str("/read-online/");
	url.push_str(&chapter_id);

	let result = Request::new(&url, HttpMethod::Get).string()?;

	// create base image url
	let base_url = helper::string_between(&result, "vm.CurPathName = \"", "\";", 0);
	let title_uri = helper::string_between(&result, "vm.IndexName = \"", "\";", 0);

	let chapter = parse(helper::string_between(&result, "vm.CurChapter = ", "};", 1).as_bytes())?
		.as_object()?;

	let directory = match chapter.get("Directory").as_string() {
		Ok(title) => title.read(),
		Err(_) => String::new(),
	};

	let mut base_path = String::from("https://");
	base_path.push_str(&base_url);
	base_path.push_str("/manga/");
	base_path.push_str(&title_uri);
	base_path.push('/');
	if !directory.is_empty() {
		base_path.push_str(&directory);
		base_path.push('/');
	}
	base_path.push_str(&helper::chapter_image(
		&chapter.get("Chapter").as_string()?.read(),
		true,
	));

	let page_count = chapter.get("Page").as_int().unwrap_or(0);

	let mut pages: Vec<Page> = Vec::with_capacity(page_count as usize);

	for i in 0..page_count {
		// pad page index to length 3 (e.g. 45 -> "046")
		let mut vec: Vec<u8> = Vec::new();
		let mut num = i + 1;
		loop {
			vec.insert(0, (num % 10) as u8 + b'0');
			num /= 10;
			if num < 1 {
				break;
			}
		}
		while vec.len() < 3 {
			vec.insert(0, b'0');
		}

		let mut page_url = base_path.clone();
		page_url.push('-');
		page_url.push_str(&String::from_utf8(vec).unwrap_or_else(|_| String::from("000")));
		page_url.push_str(".png");

		pages.push(Page {
			index: i as i32,
			url: page_url,
			..Default::default()
		})
	}

	Ok(pages)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let url_without_protocol = &url[8..]; // remove "https://"
	let end = match url_without_protocol.find('/') {
		Some(i) => i + 1,
		None => url_without_protocol.len(),
	};
	let url_without_domain = &url_without_protocol[end..]; // remove url host

	if url.starts_with("manga/") {
		// ex: https://mangasee123.com/manga/Kanojo-Okarishimasu
		//     https://manga4life.com/manga/Kanojo-Okarishimasu

		let id = url_without_domain
			.strip_prefix("manga/")
			.unwrap_or_default(); // remove "manga/"
		let id_end = match id.find('/') {
			Some(i) => i,
			None => id.len(),
		};
		let manga_id = &id[..id_end];
		let manga = get_manga_details(String::from(manga_id))?;

		return Ok(DeepLink {
			manga: Some(manga),
			chapter: None,
		});
	} else if url_without_domain.starts_with("read-online/") {
		// ex: https://manga4life.com/read-online/Kanojo-Okarishimasu-chapter-232.html

		let id = url_without_domain
			.strip_prefix("read-online/")
			.unwrap_or_default(); // remove "read-online/"
		let id_end = match id.find("-chapter") {
			Some(i) => i,
			None => id.len(),
		};
		let manga_id = &id[..id_end];
		let manga = get_manga_details(String::from(manga_id))?;
		let chapter = Chapter {
			id: String::from(id),
			url,
			..Default::default()
		};

		return Ok(DeepLink {
			manga: Some(manga),
			chapter: Some(chapter),
		});
	}

	Err(aidoku::error::AidokuError {
		reason: aidoku::error::AidokuErrorKind::Unimplemented,
	})
}
