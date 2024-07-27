use aidoku::{
	error::{AidokuError, AidokuErrorKind},
	prelude::format,
	std::{html::Node, String, Vec},
	MangaContentRating, MangaStatus, MangaViewer,
};

use crate::BASE_URL;

/// Converts `<br>` into newlines.
pub fn text_with_newlines(node: Node) -> String {
	let html = node.html().read();
	if !String::from(html.trim()).is_empty() {
		Node::new_fragment(
			node.html()
				.read()
				.replace("<br>", "{{ .LINEBREAK }}")
				.as_bytes(),
		)
		.expect("Failed to create new fragment")
		.text()
		.read()
		.replace("{{ .LINEBREAK }}", "\n")
	} else {
		String::new()
	}
}

/// Returns the ID of a manga from a URL.
pub fn get_manga_id(url: &String) -> Result<String, AidokuError> {
	// Asura Scans appends a random string at the end of each series slug
	// The random string is not necessary, but we must leave the trailing '-' else
	// the url will break
	// Example Url: https://asuracomic.net/series/swordmasters-youngest-son-cb22671f
	// Example Url: https://asuracomic.net/series/swordmasters-youngest-son-cb22671f?blahblah
	// Example Url: https://asuracomic.net/series/swordmasters-youngest-son-cb22671f/chapter/1
	// parse "swordmasters-youngest-son-" from the url

	// Split the URL to ignore query parameters
	let path = url.split('?').next().unwrap_or("");

	// Find the segment containing "series" and get the next segment
	let mut segments = path.split('/');
	while let Some(segment) = segments.next() {
		if segment == "series" {
			if let Some(manga_segment) = segments.next() {
				if let Some(pos) = manga_segment.rfind('-') {
					// We want to keep the trailing '-' in the id
					if let Some(id) = manga_segment.get(0..pos + 1) {
						return Ok(String::from(id));
					}
				}
			}
		}
	}

	Err(AidokuError {
		reason: AidokuErrorKind::Unimplemented,
	})
}

/// Returns the chapter ID of a chapter from a URL.
pub fn get_chapter_id(url: &String) -> Result<String, AidokuError> {
	// Asura Scans appends a random string at the end of each series slug
	// The random string is not necessary, but we must leave the trailing '-' else
	// the url will break
	// Example Url: https://asuracomic.net/series/swordmasters-youngest-son-cb22671f/chapter/1
	// Example Url: https://asuracomic.net/series/swordmasters-youngest-son-cb22671f/chapter/1?blahblah
	// parse "1" from the url

	// Split the URL to ignore query parameters
	let path = url.split('?').next().unwrap_or("");

	// Find the segment containing "chapter" and get the next segment
	let mut segments = path.split('/');
	while let Some(segment) = segments.next() {
		if segment == "chapter" {
			if let Some(chapter_segment) = segments.next() {
				// We want to keep the chapter ID without trailing characters
				if let Some(end_pos) = chapter_segment.find(|c: char| !c.is_numeric()) {
					let chapter_id = &chapter_segment[0..end_pos];
					return Ok(String::from(chapter_id));
				} else {
					return Ok(String::from(chapter_segment));
				}
			}
		}
	}

	Err(AidokuError {
		reason: AidokuErrorKind::Unimplemented,
	})
}

/// Returns full URL of a manga from a manga ID.
pub fn get_manga_url(manga_id: &String) -> String {
	format!("{BASE_URL}/series/{manga_id}")
}

/// Returns full URL of a chapter from a chapter ID and manga ID.
pub fn get_chapter_url(chapter_id: &String, manga_id: &String) -> String {
	format!("{BASE_URL}/series/{manga_id}/chapter/{chapter_id}")
}
