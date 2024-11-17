use aidoku::{error::Result, prelude::format, std::String};

use crate::BASE_URL;

/// Returns the ID of a manga from a URL.
pub fn get_manga_id(url: &str) -> String {
	// Two Formats
	// https://demonicscans.org/title/Overgeared/chapter/1/2024090306
	// https://demonicscans.org/manga/Overgeared
	// For the chapter format it seems as if the ending part is <year><month><day><hour> where hour is 12hr time in UTC

	let id_with_suffix = url
		.split("/manga/")
		.last()
		.unwrap_or(url.split("/title/").last().unwrap_or(""));

	// Handle chapter suffix
	let id_without_chapter_suffix = if let Some(index) = id_with_suffix.rfind("/chapter/") {
		&id_with_suffix[..index]
	} else {
		id_with_suffix
	};

	String::from(id_without_chapter_suffix)
}

/// Returns the ID of a chapter from a URL.
pub fn get_chapter_id(url: &str) -> String {
	// Example Url: https://demonicscans.org/chaptered.php?manga=4&chapter=1
	// Example Url: /chaptered.php?manga=4&chapter=1
	// parse "/chaptered.php?manga=4&chapter=1" from the url

	match url.find("/chaptered.php") {
		Some(i) => url[i..].into(),
		None => String::from(""),
	}
}

/// Returns full URL of a manga from a manga ID.
pub fn get_manga_url(manga_id: &String) -> String {
	format!("{}/manga/{}", BASE_URL, manga_id)
}

pub fn get_chapter_url(chapter_id: &String) -> Result<String> {
	Ok(format!("{}{}", BASE_URL, chapter_id))
}
