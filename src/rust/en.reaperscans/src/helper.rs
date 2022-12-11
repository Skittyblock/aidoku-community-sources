use aidoku::{
	prelude::format,
	std::{html::Node, String, Vec},
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

/// Returns the ID of a manga from a URL.
// *NOTE: This was written by GitHub Copilot.
pub fn get_manga_id(url: String) -> String {
	// Example Url: https://reaperscans.com/comics/4921-demonic-emperor/chapters/64343350-chapter-324
	// parse "4921-demonic-emperor" from the url

	if url.contains("comics") {
		// Split the url by "/"
		let split_url = url.split("/").collect::<Vec<&str>>();
		// Get the index of "comics"
		let comics_index = split_url.iter().position(|&r| r == "comics").unwrap();
		// Get the index of the manga id
		let manga_id_index = comics_index + 1;
		// Get the manga id
		let manga_id = split_url[manga_id_index];
		// Return the manga id
		String::from(manga_id)
	} else {
		// Return an empty string
		String::new()
	}
}

/// Returns the ID of a chapter from a URL.
// *NOTE: This was written by GitHub Copilot.
pub fn get_chapter_id(url: String) -> String {
	// Example Url: https://reaperscans.com/comics/4921-demonic-emperor/chapters/64343350-chapter-324
	// parse "64343350-chapter-324" from the url

	if url.contains("chapters") {
		// Split the url by "/"
		let split_url = url.split("/").collect::<Vec<&str>>();
		// Get the index of "chapters"
		let chapters_index = split_url.iter().position(|&r| r == "chapters").unwrap();
		// Get the index of the chapter id
		let chapter_id_index = chapters_index + 1;
		// Get the chapter id
		let chapter_id = split_url[chapter_id_index];
		// Return the chapter id
		String::from(chapter_id)
	} else {
		// Return an empty string
		String::new()
	}
}
