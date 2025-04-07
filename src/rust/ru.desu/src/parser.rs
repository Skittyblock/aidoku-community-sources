extern crate alloc;
use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	prelude::*,
	std::{defaults::defaults_get, ObjectRef, String, StringRef, Vec},
	Chapter, DeepLink, Manga, MangaContentRating, MangaStatus, MangaViewer, Page,
};

use crate::get_manga_details;
use aidoku::std::ArrayRef;
use alloc::string::ToString;

/*macro_rules! debug {
	($($arg:tt)*) => {{
		println!("ru.desu:: {}:{}: {}", file!(), line!(), format!($($arg)*))
	}};
}
pub(crate) use debug;*/

pub fn parse_manga_item(manga_obj: ObjectRef, skip_authors: bool) -> Result<Manga> {
	let eng_title = defaults_get("eng_title")
		.and_then(|value| value.as_bool())
		.unwrap_or(false);

	let id = manga_obj.get("id").as_int()?.to_string();
	let title = manga_obj
		.get(if eng_title { "name" } else { "russian" })
		.as_string()?
		.read();
	let url = manga_obj.get("url").as_string()?.read();
	let read_mode = manga_obj.get("reading").as_string()?.read();
	let age_limit = manga_obj.get("age_limit").as_string()?.read();
	let status = manga_obj.get("status").as_string()?.read();
	let description = manga_obj.get("description").as_string()?.read();
	let cover = manga_obj
		.get("image")
		.as_object()?
		.get("original")
		.as_string()?
		.read();
	let kind = manga_obj.get("kind").as_string()?.read();
	let mut categories = Vec::new();
	match manga_obj.get("genres").as_array() {
		Ok(gen_arr) => {
			for genre_obj in gen_arr {
				categories.push(genre_obj.as_object()?.get("text").as_string()?.read());
			}
		}
		Err(_) => {
			categories = manga_obj
				.get("genres")
				.as_string()?
				.read()
				.split(", ")
				.map(|s| s.to_string())
				.collect();
		}
	}

	let mut authors = Vec::new();
	if !skip_authors {
		if let Ok(author_obj_list) = manga_obj.get("authors").as_array() {
			for author_obj in author_obj_list {
				let author_name = author_obj
					.as_object()?
					.get("people_name")
					.as_string()
					.unwrap_or(StringRef::default())
					.read();
				if !author_name.is_empty() {
					authors.push(author_name);
				}
			}
		}
	}

	Ok(Manga {
		id,
		title,
		author: authors.join(", "),
		cover,
		description,
		url,
		categories,
		nsfw: match age_limit.as_str() {
			// "no" if no age limit. I guess safe by default is ok...
			"18_plus" => MangaContentRating::Nsfw,
			"16_plus" => MangaContentRating::Suggestive,
			_ => MangaContentRating::Safe,
		},
		status: match status.as_str() {
			// looks like they don't have hiatus status and so on
			"ongoing" => MangaStatus::Ongoing,
			"released" => MangaStatus::Completed,
			_ => MangaStatus::Unknown,
		},
		viewer: match kind.as_str() {
			// since they can set read_mode to RTL even for manhwa/manhua I decided to do this
			"manhwa" | "manhua" => MangaViewer::Scroll,
			_ => match read_mode.as_str() {
				"right-to-left" => MangaViewer::Rtl,
				"left-to-right" => MangaViewer::Ltr,
				"top-to-bottom" => MangaViewer::Scroll,
				_ => MangaViewer::Rtl,
			},
		},
		..Default::default()
	})
}

pub fn parse_manga_array(array_obj: ArrayRef, skip_authors: bool) -> Result<Vec<Manga>> {
	let mut mangas = Vec::new();
	for item in array_obj {
		mangas.push(parse_manga_item(item.as_object()?, skip_authors)?);
	}

	Ok(mangas)
}

fn parse_chapter(url: String, chapter_obj: ObjectRef) -> Result<Chapter> {
	let id = match chapter_obj.get("id").as_int() {
		Ok(id) => id.to_string(),
		// they have some cases when they return id as string :/
		Err(_) => chapter_obj.get("id").as_string()?.read(),
	};
	let title = chapter_obj
		.get("title")
		.as_string()
		.unwrap_or_default()
		.read();
	let vol = chapter_obj.get("vol").as_float().unwrap_or(-1.0) as f32;
	let ch = chapter_obj.get("ch").as_float().unwrap_or(0.0) as f32;
	let date_updated = chapter_obj.get("date").as_float().unwrap_or(-1.0);

	Ok(Chapter {
		id,
		title,
		chapter: ch,
		volume: vol,
		date_updated,
		url: format!("{}/vol{}/ch{}/rus", url, vol, ch),
		lang: String::from("ru"),
		..Default::default()
	})
}

pub fn parse_chapters(manga_obj: ObjectRef) -> Result<Vec<Chapter>> {
	let url = manga_obj.get("url").as_string()?.read(); // required for url in Chapter object
	let list = manga_obj
		.get("chapters")
		.as_object()?
		.get("list")
		.as_array()?;
	let mut chapters = Vec::new();
	for chapter_obj in list {
		let chapter = parse_chapter(url.clone(), chapter_obj.as_object()?)?;
		chapters.push(chapter);
	}

	Ok(chapters)
}

fn parse_page(idx: i32, page_obj: ObjectRef) -> Result<Page> {
	let index = page_obj
		.get("index")
		.as_int()
		.map(|v| v as i32)
		.unwrap_or(idx + 1);

	let url = page_obj.get("img").as_string()?.read();

	Ok(Page {
		index,
		url,
		..Default::default()
	})
}

pub fn parse_pages_list(manga_obj: ObjectRef) -> Result<Vec<Page>> {
	let pages_raw = manga_obj
		.get("pages")
		.as_object()?
		.get("list")
		.as_array()?
		.enumerate();

	let mut pages = Vec::new();
	for (idx, page_obj) in pages_raw {
		pages.push(parse_page(idx as i32, page_obj.as_object()?)?);
	}

	Ok(pages)
}

pub fn parse_incoming_url(url: &str) -> Result<DeepLink> {
	let manga_id = url
		.find("://")
		.map(|start_index| start_index + 3)
		.map(|path_start| &url[path_start..])
		.map(|path| path.split('/'))
		.and_then(|segments| {
			segments
				.skip_while(|&s| s == "manga" || s == "api")
				.find(|s| s.contains('.'))
		})
		.map(|target_segment| target_segment.split('.'))
		.and_then(|mut parts| parts.next_back())
		.map(|s| s.to_string())
		.ok_or(AidokuError {
			reason: AidokuErrorKind::Unimplemented,
		});

	Ok(DeepLink {
		manga: Some(get_manga_details(manga_id?)?),
		chapter: None,
	})
}
