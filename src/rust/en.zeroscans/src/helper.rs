use aidoku::{
	error::{AidokuError, Result},
	prelude::format,
	std::{
		current_date,
		net::{HttpMethod, Request},
		String, Vec,
	},
	Manga, MangaStatus, MangaViewer,
};

use crate::BASE_URL;

// Cache all the comics to avoid making multiple requests.
static mut ALL_COMICS: Option<Result<Vec<Manga>>> = None;

pub fn all_comics() -> Result<Vec<Manga>> {
	unsafe {
		if ALL_COMICS.is_some() {
			return ALL_COMICS.clone().expect("Failed to load cached comics");
		}
	}

	let url = format!("{}/swordflake/comics", BASE_URL);

	let json = Request::new(url, HttpMethod::Get).json()?.as_object()?;
	let comics = json.get("data").as_object()?.get("comics").as_array()?;

	let mut manga: Vec<Manga> = Vec::new();

	for comic in comics {
		let comic = comic.as_object()?;

		let title = comic.get("name").as_string()?.read();
		let id = comic.get("id").as_int()?;
		let slug = comic.get("slug").as_string()?.read();
		let url = format!("{}/comics/{}", BASE_URL, slug);
		let summary = comic.get("summary").as_string()?.read();
		let status = {
			let parsed_value = comic
				.get("statuses")
				.as_array()?
				.get(0)
				.as_object()?
				.get("name")
				.as_string()?
				.read();

			match parsed_value.as_str() {
				"New" => MangaStatus::Unknown,
				"Ongoing" => MangaStatus::Ongoing,
				"Completed" => MangaStatus::Completed,
				"Dropped" => MangaStatus::Cancelled,
				"Hiatus" => MangaStatus::Hiatus,
				_ => MangaStatus::Unknown,
			}
		};

		let genres: Vec<String> = comic
			.get("genres")
			.as_array()?
			.map(|genre| {
				genre
					.as_object()
					.unwrap_or_default()
					.get("name")
					.as_string()
					.unwrap_or_default()
					.read()
			})
			.collect();

		let cover = comic
			.get("cover")
			.as_object()?
			.get("horizontal")
			.as_string()?
			.read();

		manga.push(Manga {
			// Embedding the int id in the slug because we need both them
			// in different situations, wish there was a better way.
			// use the get_identifiers function to extract the id and slug
			id: format!("[<{}>]{}", id, slug),
			cover,
			title,
			description: summary,
			url,
			categories: genres,
			status,
			viewer: MangaViewer::Scroll,
			..Default::default()
		});
	}

	unsafe {
		ALL_COMICS = Some(Ok(manga.clone()));
	}

	Ok(manga)
}

/// Returns the ID and slug of a manga from a combined ID.
pub fn get_identifiers(string: &str) -> Result<(i32, String)> {
	let parts = string.split(">]").collect::<Vec<&str>>();

	// Handle edge case for comics that have not been migrated to the new id logic
	// Old id's were just the slug, which breaks this function as it expects the
	// new id format which is `[<id>]slug`
	if parts.len() != 2 {
		return Err(AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		});
	}

	let id = parts[0].replace("[<", "").parse::<i32>().unwrap_or(0);
	let slug = String::from(parts[1]);

	Ok((id, slug))
}

/// Given a valid url parse the slug and chapter id.
pub fn parse_url(url: &str) -> Option<(String, Option<String>)> {
	if let Some(start_idx) = url.find("comics/") {
		let remaining = &url[start_idx + "comics/".len()..];
		let parts: Vec<&str> = remaining.split('/').collect();

		if let Some(slug) = parts.first() {
			let chapter_id = parts.get(1).map(|id| format!("{}", id));
			return Some((format!("{}", slug), chapter_id));
		}
	}

	None
}

/// Convert a `time_ago` string to a `f64` date.
pub fn get_date(time_ago: String) -> f64 {
	let cleaned_time_ago = String::from(time_ago.replace("ago", "").trim());

	let number = cleaned_time_ago
		.split_whitespace()
		.next()
		.unwrap_or("")
		.parse::<f64>()
		.unwrap_or(0.0);

	match cleaned_time_ago
		.to_uppercase()
		.split_whitespace()
		.last()
		.unwrap_or("")
	{
		"YEAR" | "YEARS" => current_date() - (number * 60.0 * 60.0 * 24.0 * 365.0),
		"MONTH" | "MONTHS" => current_date() - (number * 60.0 * 60.0 * 24.0 * 30.0),
		"WEEK" | "WEEKS" => current_date() - (number * 60.0 * 60.0 * 24.0 * 7.0),
		"DAY" | "DAYS" => current_date() - (number * 60.0 * 60.0 * 24.0),
		_ => current_date(),
	}
}
