#![no_std]
mod helper;
mod languages;
extern crate alloc;
use aidoku::{
	error::{AidokuError, Result},
	prelude::*,
	std::{defaults::defaults_get, String, Vec},
	Chapter, DeepLink, Filter, Manga, MangaPageResult, Page,
};
use alloc::vec;

#[get_manga_list]
fn get_manga_list(_: Vec<Filter>, _: i32) -> Result<MangaPageResult> {
	let mut manga: Vec<Manga> = Vec::with_capacity(6);
	if let Ok(languages) = defaults_get("languages").as_array() {
		for lang in languages {
			manga.push(match lang.as_string()?.read().as_str() {
				"en" => languages::en::comic_info(),
				"es" => languages::es::comic_info(),
				"fr" => languages::fr::comic_info(),
				"ko" => languages::ko::comic_info(),
				"ru" => languages::ru::comic_info(),
				"zh" => languages::zh::comic_info(),
				_ => continue,
			})
		}
	} else {
		manga = vec![
			languages::en::comic_info(),
			languages::es::comic_info(),
			languages::fr::comic_info(),
			languages::ko::comic_info(),
			languages::ru::comic_info(),
			languages::zh::comic_info(),
		];
	}

	Ok(MangaPageResult {
		manga,
		has_more: false,
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	match id.as_str() {
		"multi.xkcd.en" => Ok(languages::en::comic_info()),
		"multi.xkcd.es" => Ok(languages::es::comic_info()),
		"multi.xkcd.fr" => Ok(languages::fr::comic_info()),
		"multi.xkcd.ko" => Ok(languages::ko::comic_info()),
		"multi.xkcd.ru" => Ok(languages::ru::comic_info()),
		"multi.xkcd.zh" => Ok(languages::zh::comic_info()),
		_ => Err(AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		}),
	}
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	match id.as_str() {
		"multi.xkcd.en" => languages::en::get_chapter_list(),
		"multi.xkcd.es" => languages::es::get_chapter_list(),
		"multi.xkcd.fr" => languages::fr::get_chapter_list(),
		"multi.xkcd.ko" => languages::ko::get_chapter_list(),
		"multi.xkcd.ru" => languages::ru::get_chapter_list(),
		"multi.xkcd.zh" => languages::zh::get_chapter_list(),
		_ => Err(AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		}),
	}
}

#[get_page_list]
fn get_page_list(manga_id: String, id: String) -> Result<Vec<Page>> {
	match manga_id.as_str() {
		"multi.xkcd.en" => languages::en::get_page_list(id),
		"multi.xkcd.es" => languages::es::get_page_list(id),
		"multi.xkcd.fr" => languages::fr::get_page_list(id),
		"multi.xkcd.ko" => languages::ko::get_page_list(id),
		"multi.xkcd.ru" => languages::ru::get_page_list(id),
		"multi.xkcd.zh" => languages::zh::get_page_list(id),
		_ => Err(AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		}),
	}
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let split = url
		.split('/')
		.filter_map(|val| {
			if val.is_empty() {
				None
			} else {
				Some(String::from(val))
			}
		})
		.collect::<Vec<_>>();
	// https://xkcd.tw/1321
	// ['https:', 'xkcd.tw', '1321']
	let manga = Some(match split[1].as_str() {
		"xkcd.com" => languages::en::comic_info(),
		"es.xkcd.com" => languages::es::comic_info(),
		"xkcd.lapin.org" => languages::fr::comic_info(),
		"xkcdko.com" => languages::ko::comic_info(),
		"xkcd.ru" => languages::ru::comic_info(),
		"xkcd.tw" => languages::zh::comic_info(),
		_ => {
			return Err(AidokuError {
				reason: aidoku::error::AidokuErrorKind::Unimplemented,
			})
		}
	});

	let chapter = match split.last() {
		Some(value) => {
			match split[1].as_str() {
				// Numeric IDs
				"xkcd.com" | "xkcdko.com" | "xkcd.ru" | "xkcd.tw" => match value.parse::<i32>() {
					Ok(chapter) => Some(Chapter {
						id: String::from(value),
						chapter: chapter as f32,
						url,
						..Default::default()
					}),
					Err(_) => None,
				},
				// Numeric IDs but slightly special
				"xkcd.lapin.org" => {
					let id = &value[value.find('=').unwrap_or(0) + 1..];
					match id.parse::<i32>() {
						Ok(chapter) => Some(Chapter {
							id: String::from(value),
							chapter: chapter as f32,
							url,
							..Default::default()
						}),
						Err(_) => None,
					}
				}
				"es.xkcd.com" => {
					if url.contains("strips") {
						Some(Chapter {
							id: String::from(value),
							url,
							..Default::default()
						})
					} else {
						None
					}
				}
				_ => {
					return Err(AidokuError {
						reason: aidoku::error::AidokuErrorKind::Unimplemented,
					})
				}
			}
		}
		None => None,
	};

	Ok(DeepLink { manga, chapter })
}
