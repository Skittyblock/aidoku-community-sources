use aidoku::{
	prelude::format,
	std::{html::Node, String},
};

use crate::BASE_URL;

/// Returns the ID of a manga from a URL.
pub fn get_manga_id(url: &str) -> String {
  // Two Formats
  // https://demonicscans.org/title/Overgeared/chapter/1/2024090306
  // https://demonicscans.org/manga/Overgeared
  // For the chapter format it seems as if the ending part is <year><month><day><hour> where hour is 12hr time in UTC

	let id_with_suffix= url.split("/manga/").last().unwrap_or(url.split("/title/").last().unwrap_or(""));

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
	format!("{}/manga/{}", BASE_URL, manga_id)
}

/// Returns full URL of a chapter from a chapter ID.
pub fn get_chapter_url(chapter_id: &String, manga_id: &String) -> String {
	// Append the `-VA` suffix to the chapter id to get the proper url.
	format!("{}/manga/{}/chapter/{}-VA", BASE_URL, manga_id, chapter_id)
}
