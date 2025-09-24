use crate::{constants::SEARCH_OFFSET_STEP, wrappers::WNode};
use aidoku::std::defaults::defaults_get;
use aidoku::{
	error::{AidokuError, AidokuErrorKind, NodeError, Result},
	prelude::*,
	std::net::{HttpMethod, Request},
	Manga, MangaPageResult,
};
use alloc::{
	string::{String, ToString},
	vec::Vec,
};

pub fn get_base_url() -> String {
	defaults_get("baseUrl")
		.and_then(|x| x.as_string())
		.unwrap_or_default()
		.to_string()
		.trim()
		.trim_end_matches('/')
		.to_string()
}

pub fn get_base_search_url() -> String {
	format!("{}/{}", get_base_url(), "search/advancedResults?")
}

pub fn get_html(url: &str) -> Result<WNode> {
	Request::new(url, HttpMethod::Get)
		.header("Referer", "https://www.google.com/")
		.html()
		.map(WNode::from_node)
}

pub fn get_manga_url(id: &str) -> String {
	format!("{}/{}", get_base_url(), id)
}

pub fn create_manga_page_result(mangas: Vec<Manga>) -> MangaPageResult {
	let has_more = mangas.len() == SEARCH_OFFSET_STEP as usize;
	MangaPageResult {
		manga: mangas,
		has_more,
	}
}

pub fn get_chapter_url(manga_id: &str, chapter_id: &str) -> String {
	// mtr is 18+ skip
	format!("{}/{}/{}?mtr=true", get_base_url(), manga_id, chapter_id)
}

pub fn create_parsing_error() -> AidokuError {
	AidokuError {
		reason: AidokuErrorKind::NodeError(NodeError::ParseError),
	}
}
