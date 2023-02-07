use aidoku::{
	std::{current_date, html::Node, net::Request, Vec},
	std::{net::HttpMethod, String},
	MangaStatus,
};

use crate::BASE_URL;

pub fn get_chapter_number(id: String) -> f32 {
	let values: Vec<&str> = id.split('/').collect();
	values[5].parse::<f32>().unwrap()
}

pub fn get_manga_id(raw_id: String) -> String {
	if raw_id.contains("all-pages") {
		let mut m_id = String::new();
		let first_part = &raw_id[7..];
		let last_part = &first_part[..first_part.find('/').unwrap_or(first_part.len())];
		m_id.push_str(&raw_id[..7]);
		m_id.push_str(last_part);
		m_id
	} else {
		raw_id
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

pub fn manga_status(status: String) -> MangaStatus {
	match status.as_str() {
		"ONGOING" => MangaStatus::Ongoing,
		"COMPLETED" => MangaStatus::Completed,
		"HIATUS" => MangaStatus::Hiatus,
		"CANCELLED" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	}
}

pub fn get_page_number(id: String) -> i32 {
	let html = Request::new(id.as_str(), HttpMethod::Get).html();
	let mut url = String::new();
	for manga in html.select(".ui.pagination.menu a").array() {
		let manga_node = manga.as_node();
		let last_page_string = manga_node.text().read();
		if last_page_string == "Last" {
			url = String::from(manga_node.attr("href").read().as_str());
		}
	}
	let values: Vec<&str> = url.split('/').collect();
	String::from(values[2]).parse::<i32>().unwrap_or(1)
}

pub fn get_full_url(id: String) -> String {
	if id.contains("readm.org") {
		id
	} else {
		String::from(BASE_URL) + id.as_str()
	}
}

pub fn genres() -> [&'static str; 45] {
	[
		"all",
		"action",
		"adventure",
		"comedy",
		"cooking",
		"doujinshi",
		"drama",
		"ecchi",
		"fantasy",
		"gender Bender",
		"harem",
		"historical",
		"horror",
		"isekai",
		"josei",
		"lolicon",
		"magic",
		"manga",
		"manhua",
		"manhwa",
		"martial Arts",
		"mature",
		"mecha",
		"mind Game",
		"mystery",
		"none",
		"one Shot",
		"psychological",
		"recarnation",
		"romance",
		"school Life",
		"sci-fi",
		"seinen",
		"shotacon",
		"shoujo Ai",
		"shoujo",
		"shounen Ai",
		"shounen",
		"slice of Life",
		"sports",
		"supernatural",
		"tragedy",
		"uncategorized",
		"yaoi",
		"yuri",
	]
}

pub fn get_image_src(node: Node) -> String {
	let image = node.select("img").first().attr("src").read();
	if image.starts_with("data") || image.is_empty() {
		node.select("img").first().attr("data-src").read()
	} else {
		image
	}
}
