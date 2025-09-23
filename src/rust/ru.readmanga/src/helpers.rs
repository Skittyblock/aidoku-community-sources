use aidoku::{
	error::{AidokuError, AidokuErrorKind, NodeError, Result},
	prelude::*,
	std::net::{HttpMethod, Request},
	Manga, MangaPageResult,
};
use alloc::{string::String, vec::Vec};

use crate::{
	constants::{BASE_URL, SEARCH_OFFSET_STEP},
	wrappers::WNode,
};

pub fn get_html(url: &str) -> Result<WNode> {
	Request::new(url, HttpMethod::Get)
		.header("Referer", "https://www.google.com/")
		.html()
		.map(WNode::from_node)
}

pub fn get_manga_url(id: &str) -> String {
	format!("{}/{}", BASE_URL, id)
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
	format!("{BASE_URL}/{manga_id}/{chapter_id}?mtr=true")
}

pub fn create_parsing_error() -> AidokuError {
	AidokuError {
		reason: AidokuErrorKind::NodeError(NodeError::ParseError),
	}
}
