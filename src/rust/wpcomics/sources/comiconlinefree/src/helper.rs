use aidoku::{
	prelude::format,
	std::{String, StringRef, Vec},
};
use wpcomics_template::helper::urlencode;

pub fn listing_mapping(listing: String) -> String {
	String::from(match listing.as_str() {
		"Hot" => "hot-comic",
		"Popular" => "popular-comic",
		_ => "",
	})
}

pub fn convert_time(time_ago: String) -> f64 {
	let time_object = StringRef::from(time_ago).0;
	time_object
		.as_date("MM/dd/yy", Some("en_US"), None)
		.unwrap_or(0.0)
}

pub fn get_search_url(
	base_url: String,
	query: String,
	include: Vec<String>,
	exclude: Vec<String>,
	completed: String,
	page: i32,
) -> String {
	if query.is_empty() && completed.is_empty() && include.is_empty() && exclude.is_empty() {
		String::from("https://comiconlinefree.net")
	} else {
		format!(
			"{base_url}/advanced-search?key={query}&wg={}&wog={}&status={completed}&page={page}",
			urlencode(include.join(",")),
			urlencode(exclude.join(",")),
		)
	}
}
