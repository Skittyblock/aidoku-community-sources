use aidoku::{prelude::format, std::String};

pub fn listing_map(listing: String) -> String {
	let url: &str = match listing.as_str() {
		"Popular" => "popular-comic",
		"Hot" => "hot-comic",
		"Completed" => "completed-comic",
		"Ongoing" => "ongoing-comic",
		"New" => "new-comic",
		_ => "popular-comic",
	};
	String::from(url)
}

// MARK: Other utilities
pub fn get_search_url(base_url: &str, query: &str, genre: &str, page: i32) -> String {
	if !query.is_empty() {
		format!("{base_url}/search-comic?key={query}&page={page}")
	} else if !genre.is_empty() {
		format!("{base_url}/{genre}-comic?page={page}")
	} else {
    format!("{base_url}/popular-comic")
	}
}

pub fn title_transformer(title: String) -> String {
	title
		.trim_start_matches("Read ")
		.trim_end_matches(" Comics Online for Free")
		.into()
}

pub fn chapter_title_transformer(title: String, chapter_title: String, _volume: f32, _chapter: f32) -> String {
  chapter_title.replace(&title, "").replace("_", " ").into()
}

pub fn chapter_to_vol_chap(_title: String, _chapter_title: String) -> (f32, f32) {
  (-1.0, -1.0)
}