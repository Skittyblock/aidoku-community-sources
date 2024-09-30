use aidoku::{
	prelude::format,
	std::{html::Node, String},
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
pub fn get_manga_id(url: &str) -> String {
	// NOTE: From my limited testing every manga seems to have a suffix of `-VA54`
	// But it looks like only `-VA` is required we can remove it for the id and
	// add it back for the url
	//
	// Example Url: https://demonreader.org/manga/Skeleton-Soldier-VA54
	// Example Url: https://demonreader.org/manga/Skeleton-Soldier/chapter/1-VA54
	// parse "Skeleton-Soldier" from the url

	let id_with_suffix = url.split("/manga/").last().unwrap_or("");

	let id_without_suffix = if let Some(index) = id_with_suffix.rfind("-VA") {
		&id_with_suffix[..index]
	} else {
		id_with_suffix
	};

	// Handle additional suffixes like "/chapter/1-VA54"
	let id_without_chapter_suffix = if let Some(index) = id_without_suffix.rfind("/chapter/") {
		&id_without_suffix[..index]
	} else {
		id_without_suffix
	};

	String::from(id_without_chapter_suffix)
}

/// Returns the ID of a chapter from a URL.
pub fn get_chapter_id(url: &str) -> String {
	// NOTE: From my limited testing every chapter seems to have a suffix of `-VA54`
	// But it looks like only `-VA` is required we can remove it for the id and
	// add it back for the url
	//
	// Example Url: https://demonreader.org/manga/Skeleton-Soldier/chapter/1-VA54
	// parse "1" from the url

	let id_with_suffix = url.split("/chapter/").last().unwrap_or("");

	let id_without_suffix = if let Some(index) = id_with_suffix.rfind("-VA") {
		&id_with_suffix[..index]
	} else {
		id_with_suffix
	};

	String::from(id_without_suffix)
}

/// Returns full URL of a manga from a manga ID.
pub fn get_manga_url(manga_id: &String) -> String {
	// Append the `-VA` suffix to the manga id to get the proper url.
	format!("{}/manga/{}-VA", BASE_URL, manga_id)
}

/// Returns full URL of a chapter from a chapter ID.
pub fn get_chapter_url(chapter_id: &String, manga_id: &String) -> String {
	// Append the `-VA` suffix to the chapter id to get the proper url.
	format!("{}/manga/{}/chapter/{}-VA", BASE_URL, manga_id, chapter_id)
}
