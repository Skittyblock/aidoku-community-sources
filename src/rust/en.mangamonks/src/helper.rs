use aidoku::{
	helpers::uri::QueryParameters, prelude::format, std::{current_date, html::Node, String, Vec}, MangaStatus
};

pub fn get_image_src(node: &Node, selector: &str) -> String {
	let image = node.select(selector).first().attr("src").read();
	if image.starts_with("data") || image.is_empty() {
		node.select("img").first().attr("data-src").read()
	} else {
		image
	}
}

pub fn manga_status(status: String) -> MangaStatus {
	match status.as_str() {
		"ONGOING" => MangaStatus::Ongoing,
		"COMPLETED" => MangaStatus::Completed,
		"HIATUS" => MangaStatus::Hiatus,
		"CANCELLED" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	}
}

pub fn get_date(time_ago: String) -> f64 {
	let number = time_ago
		.split_whitespace()
		.next()
		.unwrap_or("")
		.parse::<f64>()
		.unwrap_or(0.0);

	match time_ago
		.replace(" ago", "")
		.to_uppercase()
		.split_whitespace()
		.last()
		.unwrap_or("")
	{
		"YEAR" | "YEARS" => current_date() - (number * 31556926.0),
		"MONTH" | "MONTHS" => current_date() - (number * 2629743.0),
		"WEEK" | "WEEKS" => current_date() - (number * 604800.0),
		"YESTERDAY" | "DAYS" => current_date() - (number * 86400.0),
		_ => current_date(),
	}
}

pub fn parse_chapter_number(str: &str) -> f32 {
	let chapter_number = str.split(' ').nth_back(0).unwrap_or_default();
	chapter_number.parse::<f32>().unwrap_or_default()
}

pub fn get_search_url(
	base_url: &str,
	included_tags: Vec<String>,
	excluded_tags: Vec<String>,
	manga_type: String,
	status: String,
	page: i32,
) -> String {
	let mut query = QueryParameters::new();
	if !included_tags.is_empty() || !excluded_tags.is_empty() {
		if excluded_tags.is_empty() {
			for tag in included_tags {
				query.set("include[]", Some(tag.as_str()));
			}
		} else if !included_tags.is_empty() && !excluded_tags.is_empty() {
			for tag in included_tags {
				query.set("include[]", Some(tag.as_str()));
			}
			for tag in excluded_tags {
				query.set("exclude[]", Some(tag.as_str()));
			}
		} else {
			for tag in excluded_tags {
				query.set("exclude[]", Some(tag.as_str()));
			}
		}
	}
	if !manga_type.is_empty() {
		query.set("language[]", Some(manga_type.as_str()));
	}
	if !status.is_empty() {
		query.set("status[]", Some(status.as_str()));
	}
	format!("{base_url}/genre/action/{page}?term=&{query}")
}
