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
	// parse "swordmasters-youngest-son-" from the url

	if let Some(last_segment) = url.split('/').last() {
		if let Some(pos) = last_segment.rfind('-') {
			// We want to keep the trailing '-' in the id
			if let Some(id) = last_segment.get(0..pos + 1) {
				return Ok(String::from(id));
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
