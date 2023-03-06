use aidoku::{prelude::format, std::String};

// MARK: Mappings
pub fn listing_map(listing: String) -> String {
	let url: &str = match listing.as_str() {
		"Popular" => "popular-comics",
		"Hot" => "hot",
		"New" => "new-comics",
		"Completed" => "status/completed",
		"Ongoing" => "status/ongoing",
		_ => "",
	};
	String::from(url)
}

// MARK: Other utilities
pub fn get_search_url(base_url: String, query: String, genre: String, page: i32) -> String {
	if !query.is_empty() {
		format!("{base_url}/search?page={page}&keyword={query}")
	} else if !genre.is_empty() {
		format!("{base_url}/genre/{genre}?page={page}")
	} else {
		base_url
	}
}
