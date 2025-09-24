use aidoku::std::defaults::defaults_get;
use aidoku::{prelude::*, Manga, MangaPageResult, MangaStatus};
use alloc::{
	string::{String, ToString},
	vec::Vec,
};

use crate::constants::{MANGA_DIR, SEARCH_OFFSET_STEP};

pub fn show_nsfw() -> bool {
	defaults_get("showNsfw")
		.and_then(|x| x.as_bool())
		.unwrap_or_default()
}

pub fn show_only_nsfw() -> bool {
	defaults_get("showOnlyNsfw")
		.and_then(|x| x.as_bool())
		.unwrap_or_default()
}

pub fn get_base_url() -> String {
	defaults_get("baseUrl")
		.and_then(|x| x.as_string())
		.unwrap_or_default()
		.to_string()
		.trim()
		.trim_end_matches('/')
		.to_string()
}

pub fn get_manga_base_url() -> String {
	format!("{}/{}", get_base_url(), MANGA_DIR)
}

pub fn get_manga_url(id: &str) -> String {
	format!("{}/{id}", get_manga_base_url())
}

pub fn get_manga_id(url: &str) -> Option<String> {
	let split: Vec<_> = match url.find("://") {
		Some(idx) => &url[idx + 3..],
		None => url,
	}
	.split('/')
	.collect();

	let base_no_scheme: String = {
		let base = get_base_url();
		match base.find("://") {
			Some(idx) => base[idx + 3..].to_string(),
			None => base,
		}
	};

	if split.len() < 3 || split[0] != base_no_scheme.as_str() || split[1] != MANGA_DIR {
		return None;
	}

	let manga_id = split[2];
	Some(manga_id.to_string())
}

pub fn create_manga_page_result(mangas: Vec<Manga>, has_more: Option<bool>) -> MangaPageResult {
	let has_more = has_more.unwrap_or(mangas.len() == SEARCH_OFFSET_STEP as usize);
	MangaPageResult {
		manga: mangas,
		has_more,
	}
}

pub fn get_chapter_url(manga_id: &str, chapter_id: &str) -> String {
	// ?style=list is to preload all images
	format!(
		"{}/{manga_id}/{chapter_id}/?style=list",
		get_manga_base_url()
	)
}

pub fn parse_status(status_str: &str) -> MangaStatus {
	match status_str.trim() {
		"Онгоинг" => MangaStatus::Ongoing,
		"Завершен" => MangaStatus::Completed,
		"Брошено" => MangaStatus::Cancelled,
		"Заморожен" => MangaStatus::Hiatus,
		_ => MangaStatus::Unknown,
	}
}
