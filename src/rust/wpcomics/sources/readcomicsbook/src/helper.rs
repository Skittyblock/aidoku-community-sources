use aidoku::{prelude::format, std::String};

pub fn listing_mapping(listing: String) -> String {
	String::from(match listing.as_str() {
		"Ongoing" => "ongoing-comics",
		"New" => "new-comics",
		"Popular" => "popular-comics",
		_ => "",
	})
}

pub fn get_search_url(base_url: String, genre: String, page: i32) -> String {
	if !genre.is_empty() {
		return format!("{base_url}/genre/{genre}?page={page}");
	}
	format!("{base_url}/comic-updates?page={page}")
}
