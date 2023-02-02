use aidoku::{
	prelude::format,
	std::html::Node,
	std::{String, Vec},
	MangaContentRating, MangaStatus, MangaViewer,
};

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

/// Returns the status of a manga from a string.
pub fn get_manga_status(status: String) -> MangaStatus {
	match status.to_lowercase().as_str() {
		"ongoing" => MangaStatus::Ongoing,
		"completed" => MangaStatus::Completed,
		"cancelled" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	}
}

/// Returns the content rating of a manga from a vector of categories.
pub fn get_manga_content_rating(categories: Vec<String>) -> MangaContentRating {
	let mut rating = MangaContentRating::Safe;

	if !categories.is_empty() {
		if categories
			.iter()
			.any(|e| e == "Ecchi" || e == "Harem" || e == "Adult" || e == "Loli" || e == "Shota")
		{
			rating = MangaContentRating::Suggestive;
		}
		if categories
			.iter()
			.any(|e| e == "Gore" || e == "Sexual violence" || e == "Erotica")
		{
			rating = MangaContentRating::Nsfw;
		}
	}

	rating
}

/// Returns the viewer of a manga from a vector of categories.
pub fn get_manga_viewer(categories: Vec<String>) -> MangaViewer {
	let mut viewer = MangaViewer::Rtl;

	if !categories.is_empty()
		&& categories
			.iter()
			.any(|e| e == "Manhwa" || e == "Manhua" || e == "Webtoon")
	{
		viewer = MangaViewer::Scroll;
	}

	viewer
}

/// Returns the ID of a manga from a URL.
pub fn get_manga_id(url: String) -> String {
	// MangaKatana has unique numeric IDs for each manga.
	// The id is preceded by a period then a random string.
	// Example Url: https://mangakatana.com/manga/go-toubun-no-hanayome.18224
	// Example Url: https://mangakatana.com/manga/id.18224
	// parse "18224" from the url

	let mut id = String::new();

	let mut split_url = url.split(['/', '.']).collect::<Vec<&str>>();
	split_url.reverse();

	if !split_url.is_empty() {
		// I'm doing this to handle edge cases where the id is not the last part of the
		// url.
		for part in split_url {
			// If you reach a period, you have passed the id and failed to find it.
			if part == "." {
				break;
			}
			// The first part that can be parsed as a u32 is the id.
			if part.parse::<u32>().is_ok() {
				id = String::from(part);
				break;
			}
		}
	}

	id
}

/// Returns the ID of a chapter from a URL.
pub fn get_chapter_id(url: String) -> String {
	// MangaKatana has unique numeric IDs for each manga.
	// The id is preceded by a period then a random string.
	// Example Url: https://mangakatana.com/manga/go-toubun-no-hanayome.18224/c1
	// Example Url: https://mangakatana.com/manga/id.18224/c1
	// parse "c1" from the url

	let mut id = String::new();

	let mut split_url = url.split('/').collect::<Vec<&str>>();
	split_url.reverse();

	if !split_url.is_empty() {
		// I'm doing this to handle edge cases where the
		// id is not the last part of the url.
		for part in split_url {
			// If you reach a period, you have passed the id and failed to find it.
			if part == "." {
				break;
			}
			// The first part that contains a 'c' and can be parsed as a float is the id.
			if part.contains('c')
				&& part
					.split('c')
					.last()
					.expect("Failed to parse chapter id")
					.parse::<f32>()
					.is_ok()
			{
				id = String::from(part);
				break;
			}
		}
	}

	id
}

/// Returns full URL of a manga from a manga ID.
pub fn get_manga_url(manga_id: String, base_url: String) -> String {
	// MangaKatana manga urls contain a random string followed by a period then the
	// manga id. I'm setting the random string to "id" as it can be anything, then
	// appending the period & id.
	// Example manga id: 18224
	// return "https://mangakatana.com/manga/id.18224"

	format!("{}/manga/id.{}", base_url, manga_id)
}

/// Returns full URL of a chapter from a chapter ID.
pub fn get_chapter_url(chapter_id: String, manga_id: String, base_url: String) -> String {
	// MangaKatana manga urls contain a random string followed by a period then the
	// manga id. I'm setting the random string to "id" as it can be anything, then
	// appending the period & id.
	// The chapter id is the chapter number preceded by a "c".
	// Example manga id: 18224
	// Example chapter id: c1
	// return "https://mangakatana.com/manga/id.18224/c1"

	format!("{}/manga/id.{}/{}", base_url, manga_id, chapter_id)
}
