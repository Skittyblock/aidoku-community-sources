use aidoku::{prelude::format, std::String};

pub fn listing_map(listing: String) -> String {
	let url: &str = match listing.as_str() {
		"Popular" => "popular-comic",
		"Hot" => "hot-comic",
		"Completed" => "completed-comic",
		"Ongoing" => "ongoing-comic",
		_ => "",
	};
	String::from(url)
}

// MARK: Other utilities
pub fn get_search_url(base_url: String, query: String, genre: String, page: i32) -> String {
	if !query.is_empty() {
		format!("{base_url}/search-comic?keyword={query}&page={page}")
	} else if !genre.is_empty() {
		format!("{base_url}/{genre}-comic?page={page}")
	} else {
		base_url
	}
}
