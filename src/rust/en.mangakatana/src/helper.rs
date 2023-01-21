use aidoku::{
	prelude::format,
	std::{html::Node, String, Vec},
	MangaContentRating, MangaStatus, MangaViewer,
};

/// Returns an array of f32s contained within a string.
pub fn extract_f32_from_string(text: String) -> Vec<f32> {
	let mut last_char_was_digit: bool = false;
	text.chars()
		.filter(|a| {
			if (*a).is_ascii_digit() {
				last_char_was_digit = true;
				return true;
			} else if *a == '.' && last_char_was_digit || *a == '+' || *a == ' ' {
				last_char_was_digit = false;
				return true;
			}
			false
		})
		.collect::<String>()
		.split(' ')
		.filter_map(|a| a.parse::<f32>().ok())
		.collect::<Vec<f32>>()
}

/// Adds https: to the start of a URL if it is missing.
///
/// Mostly useful for URLs such as `//www.google.com` where the intent is
/// to use the current protocol.
pub fn append_protocol(url: String) -> String {
	if !url.starts_with("http") {
		return format!("{}{}", "https:", url);
	} else {
		return url;
	}
}

/// Percent-encode any non-ASCII characters in a string.
pub fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789ABCDEF".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if curr.is_ascii_alphanumeric() {
			result.push(curr);
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}
	String::from_utf8(result).unwrap_or_default()
}

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
	// parse "18224" from the url

	let mut id = String::new();

	let mut split_url = url.split(['/', '.']).collect::<Vec<&str>>();
	split_url.reverse();

	if !split_url.is_empty() {
		// I'm doing this to handle edge cases where the id is not the last part of the url.
		for part in split_url {
			// If you reach a period, you have passed the id and failed to find it.
			if part == "." {
				break;
			}
			// The first part that can be parsed as a u32 is the id.
			if part.parse::<u32>().is_ok() {
				id = String::from(part);
			}
		}
	}

	id
}

/// Returns full URL of a manga from a manga ID.
pub fn get_manga_url(manga_id: String, base_url: String) -> String {
	// MangaKatana manga urls contain a random string followed by a period then the manga id.
	// I'm setting the random string to "id" as it can be anything, then appending the period & id.
	// Example manga id: 18224
	// return "https://mangakatana.com/manga/id.18224"

	format!("{}/manga/id.{}", base_url, manga_id)
}
