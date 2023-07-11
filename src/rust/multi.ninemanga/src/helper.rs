use aidoku::{
	prelude::format,
	std::{
		current_date,
		defaults::defaults_get,
		html::Node,
		net::{HttpMethod, Request},
		String, Vec,
	},
	MangaStatus,
};

pub fn get_lang_code() -> Option<String> {
	if let Ok(languages_val) = defaults_get("languages") {
		if let Ok(languages) = languages_val.as_array() {
			if let Ok(language) = languages.get(0).as_string() {
				return Some(language.read());
			}
		}
	}
	None
}

pub fn get_manga_id(url: &str) -> String {
	url.split('/')
		.nth_back(0)
		.unwrap_or_default()
		.replace(".html", "")
}

pub fn extract_f32_from_string(chapter_title: &str, name: &str) -> f32 {
	if is_string_numeric(String::from(chapter_title)) {
		chapter_title.parse::<f32>().unwrap_or(0.0)
	} else if chapter_title.contains("vol")
		|| chapter_title.contains("Том")
		|| chapter_title.contains("Vol")
	{
		let title = chapter_title.to_lowercase();
		let text: String = if title.contains("volumen") {
			title
				.replace(&title[..title.find("volumen").unwrap() + 8], "")
				.replace("ch.", "")
		} else if title.contains("vol") {
			title
				.replace(&title[..title.find("vol.").unwrap() + 7], "")
				.replace("ch.", "")
		} else {
			title.replace(
				&chapter_title[..chapter_title.find("Том").unwrap_or(0) + 8],
				"",
			)
		};

		text.chars()
			.filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.')
			.collect::<String>()
			.split(' ')
			.collect::<Vec<&str>>()
			.into_iter()
			.map(|a| a.parse::<f32>().unwrap_or(0.0))
			.find(|a| *a > 0.0)
			.unwrap_or(0.0)
	} else {
		chapter_title
			.to_ascii_lowercase()
			.replace(&name.to_ascii_lowercase(), "")
			.replace("ch.", "")
			.replace("Ch.", "")
			.split('-')
			.last()
			.unwrap()
			.chars()
			.filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.')
			.collect::<String>()
			.split(' ')
			.collect::<Vec<&str>>()
			.into_iter()
			.map(|a| a.parse::<f32>().unwrap_or(0.0))
			.find(|a| *a > 0.0)
			.unwrap_or(0.0)
	}
}

pub fn status_from_string(status: String) -> MangaStatus {
	return match status.as_str() {
		"Ongoing"
		| "En curso"
		| "постоянный"
		| "Laufende"
		| "In corso"
		| "Em tradução"
		| "en cours" => MangaStatus::Ongoing,
		"Completed"
		| "Completado"
		| "завершенный"
		| "Abgeschlossen"
		| "Completato"
		| "Completo"
		| "Complété" => MangaStatus::Completed,
		"Hiatus" => MangaStatus::Hiatus,
		"Cancelled" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	};
}

fn is_string_numeric(str: String) -> bool {
	for c in str.chars() {
		if !c.is_numeric() {
			return false;
		}
	}
	true
}

pub fn get_date(node: Node, date_format: &str, locale: &str) -> f64 {
	let time_en = ["min", "minute", "minutes", "hour", "hours"];
	let time_es = ["minuto", "minutos", "hora", "horas"];
	let time_ru = ["минута", "минуты", "час", "часы"];
	let time_de = ["Minute", "Protokoll", "Stunde", "Stunden"];
	let time_it = ["minuto", "minuti", "ora", "ore"];
	let time_pt_br = ["minuto", "atas", "hora", "horas"];
	let time_fr = ["minute", "minutes", "heure", "heures"];
	let date_str = node.select("span").text().read();
	if time_en.iter().any(|value| date_str.contains(value))
		|| time_es.iter().any(|value| date_str.contains(value))
		|| time_ru.iter().any(|value| date_str.contains(value))
		|| time_de.iter().any(|value| date_str.contains(value))
		|| time_it.iter().any(|value| date_str.contains(value))
		|| time_pt_br.iter().any(|value| date_str.contains(value))
		|| time_fr.iter().any(|value| date_str.contains(value))
	{
		current_date()
	} else {
		node.select("span")
			.text()
			.0
			.as_date(date_format, Some(locale), None)
			.unwrap_or(-1.0)
	}
}

pub fn get_chapter_pages(base_url: &str, id: &str) -> Vec<String> {
	let mut pages: Vec<String> = Vec::new();
	if let Ok(html) = Request::new(id, HttpMethod::Get).html() {
		for page in html.select("select#page").first().select("option").array() {
			if let Ok(page_node) = page.as_node() {
				pages.push(format!("{}{}", base_url, page_node.attr("value").read()));
			}
		}
	}
	pages
}

pub fn get_search_url(
	base_url: &str,
	query: String,
	included_tags: Vec<String>,
	excluded_tags: Vec<String>,
	status: String,
	page: i32,
) -> String {
	let mut url = format!("{}/search/?name_sel=contain", base_url);
	if !query.is_empty() {
		url.push_str(&format!("&wd={}", &query.replace(' ', "+")));
	} else if !included_tags.is_empty() || !excluded_tags.is_empty() {
		if excluded_tags.is_empty() {
			url.push_str("&category_id=");
			for tag in included_tags {
				url.push_str(&format!("{}%2C", &tag));
			}
		} else if !included_tags.is_empty() && !excluded_tags.is_empty() {
			url.push_str("&category_id=");
			for tag in included_tags {
				url.push_str(&format!("{}%2C", &tag));
			}
			url.push_str("&out_category_id=");
			for tag in excluded_tags {
				url.push_str(&format!("{}%2C", &tag));
			}
		} else {
			url.push_str("&out_category_id=");
			for tag in excluded_tags {
				url.push_str(&format!("{}%2C", &tag));
			}
		}
	} else if !status.is_empty() {
		match status.as_str() {
			"yes" | "no" => url.push_str(&format!("&completed_series={}", &status)),
			_ => (),
		}
	} else {
		return format!("{}/list/New-Update/", base_url);
	}
	if !status.is_empty() {
		match status.as_str() {
			"yes" | "no" => url.push_str(&format!("&completed_series={}", &status)),
			_ => (),
		}
	}
	url.push_str(&format!("&page={}", &page));
	url.push_str("&type=high");
	url
}
