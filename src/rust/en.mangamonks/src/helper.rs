use aidoku::{
	prelude::format,
	std::{current_date, html::Node, String, Vec},
	MangaStatus,
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
	let mut url = format!("{}/genre/action/{}?term=", base_url, page);
	if !included_tags.is_empty() || !excluded_tags.is_empty() {
		if excluded_tags.is_empty() {
			for tag in included_tags {
				url.push_str(&format!("&include%5B%5D={}", tag));
			}
		} else if !included_tags.is_empty() && !excluded_tags.is_empty() {
			for tag in included_tags {
				url.push_str(&format!("&include%5B%5D={}", tag));
			}
			for tag in excluded_tags {
				url.push_str(&format!("&exclude%5B%5D={}", tag));
			}
		} else {
			for tag in excluded_tags {
				url.push_str(&format!("&exclude%5B%5D={}", tag));
			}
		}
	}
	if !manga_type.is_empty() {
		url.push_str(&format!("&language%5B%5D={}", manga_type));
	}
	if !status.is_empty() {
		url.push_str(&format!("&status%5B%5D={}", status));
	}
	url
}
