use aidoku::{
	prelude::format,
	std::{current_date, String, Vec},
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

/// Returns the ID of a manga from a URL.
// *NOTE: This was written by GitHub Copilot.
pub fn get_manga_id(url: String) -> String {
	// Example Url: https://reaperscans.com/comics/4921-demonic-emperor/chapters/64343350-chapter-324
	// parse "4921-demonic-emperor" from the url

	if url.contains("comics") {
		// Split the url by "/"
		let split_url = url.split('/').collect::<Vec<&str>>();
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
		let split_url = url.split('/').collect::<Vec<&str>>();
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

/// Returns full URL of a manga from a manga ID.
// *NOTE: This was written by GitHub Copilot.
pub fn get_manga_url(manga_id: String, base_url: String) -> String {
	// Example manga id: 4921-demonic-emperor
	// return "https://reaperscans.com/comics/4921-demonic-emperor"

	format!("{}/comics/{}", base_url, manga_id)
}

/// Returns full URL of a chapter from a chapter ID and manga ID.
// *NOTE: This was written by GitHub Copilot.
pub fn get_chapter_url(chapter_id: String, manga_id: String, base_url: String) -> String {
	// Example chapter id: 64343350-chapter-324
	// Example manga id: 4921-demonic-emperor
	// return "https://reaperscans.com/comics/4921-demonic-emperor/chapters/64343350-chapter-324"

	format!("{}/comics/{}/chapters/{}", base_url, manga_id, chapter_id)
}

pub fn get_date(time_ago: String) -> f64 {
	let cleaned_time_ago = String::from(time_ago.replace("Released", "").replace("ago", "").trim());

	let number = cleaned_time_ago
		.split_whitespace()
		.next()
		.unwrap_or("")
		.parse::<f64>()
		.unwrap_or(0.0);

	match cleaned_time_ago
		.to_uppercase()
		.split_whitespace()
		.last()
		.unwrap_or("")
	{
		"YEAR" | "YEARS" => current_date() - (number * 31556926.0),
		"MONTH" | "MONTHS" => current_date() - (number * 2629743.0),
		"WEEK" | "WEEKS" => current_date() - (number * 604800.0),
		"DAY" | "DAYS" => current_date() - (number * 86400.0),
		_ => current_date(),
	}
}
