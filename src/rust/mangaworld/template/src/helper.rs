use aidoku::{
	prelude::format,
	std::{String, Vec},
	MangaStatus,
};

pub fn manga_status(status: String) -> MangaStatus {
	match status.as_str() {
		"In corso" => MangaStatus::Ongoing,
		"Finito" => MangaStatus::Completed,
		"Droppato" => MangaStatus::Cancelled,
		"In pausa" => MangaStatus::Hiatus,
		"Cancellato" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	}
}

pub fn get_chapter_number(id: String) -> f32 {
	id.chars()
		.filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.')
		.collect::<String>()
		.split(' ')
		.collect::<Vec<&str>>()
		.into_iter()
		.map(|a| a.parse::<f32>().unwrap_or(0.0))
		.find(|a| *a > 0.0)
		.unwrap_or(0.0)
}

// generates the search, filter and homepage url
pub fn get_search_url(
	base_url: String,
	query: String,
	page: i32,
	included_tags: Vec<String>,
	status: String,
	manga_type: String,
) -> String {
	let mut url = format!("{base_url}/archive?page={page}");
	if query.is_empty() && included_tags.is_empty() && status.is_empty() && manga_type.is_empty() {
		return format!("{base_url}/archive?page={page}");
	}
	if !query.is_empty() {
		url.push_str(&format!("&keyword={}", query.replace(' ', "%20")))
	}
	if !included_tags.is_empty() {
		for tag in included_tags {
			url.push_str(&format!("&genre={}", tag));
		}
	}
	if !status.is_empty() {
		url.push_str(&format!("&status={}", status));
	}
	if !manga_type.is_empty() {
		url.push_str(&format!("&type={}", manga_type));
	}
	url
}
