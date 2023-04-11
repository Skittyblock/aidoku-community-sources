use aidoku::{error::Result, helpers::uri::*, prelude::format, std::defaults::defaults_get};
use alloc::{
	string::{String, ToString},
	vec::Vec,
};

pub const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_1_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";
pub const BASE_URL: &str = "https://simply-hentai.com";
pub const API_BASE_URL: &str = "https://api.simply-hentai.com/v3";

pub fn make_search_url(
	language: &str,
	page: i32,
	term: Option<String>,
	tags: Vec<String>,
	sort: Option<&str>,
) -> String {
	let mut query = QueryParameters::new();
	query.set("query", Some(term.unwrap_or_default().as_str()));
	query.set("sort", sort);
	query.set("page", Some(page.to_string().as_str()));
	query.set("filter[language][0]", Some(language));
	for (i, tag) in tags.iter().enumerate() {
		query.set(format!("filter[tags][{}]", i), Some(tag.to_string()));
	}
	format!("{API_BASE_URL}/search/complex?{query}")
}

pub fn get_image_quality() -> Result<String> {
	Ok(defaults_get("image_quality")?.as_string()?.read())
}
