use aidoku::{
	prelude::format,
	std::{String, Vec},
	MangaStatus,
};

pub fn manga_status(status: String) -> MangaStatus {
	match status.to_lowercase().as_str() {
		"ativo" => MangaStatus::Ongoing,
		"completo" => MangaStatus::Completed,
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
	sort_letter: String,
) -> String {
	let mut url = format!("{base_url}/mangabr?pagina={page}");
	if query.is_empty() && included_tags.is_empty() && status.is_empty() && sort_letter.is_empty() {
		return format!("{base_url}/mangas&pagina={page}");
	}
	if !query.is_empty() {
		url.push_str(&format!("&busca={}", query.replace(' ', "%20")))
	}
	if !included_tags.is_empty() {
		url.push_str("&genero=");
		for tag in included_tags {
			url.push_str(&tag);
			url.push(',');
		}
	}
	if !status.is_empty() {
		url.push_str(&format!("&status={}", status));
	}
	if !sort_letter.is_empty() {
		url.push_str(&format!("&letra={}", sort_letter));
	}
	url
}

pub fn append_domain(base_url: String, url: String) -> String {
	if url.starts_with("/mangabr") || url.starts_with("/timthumb") || url.starts_with("/mm-admin") {
		format!("{base_url}{url}")
	} else {
		url
	}
}
