use aidoku::{
	error::Result, std::defaults::defaults_get, std::html::unescape_html_entities, std::html::Node,
	std::ObjectRef, std::String, std::Vec, Chapter, Manga, MangaContentRating, MangaStatus,
	MangaViewer,
};

use crate::model::{Directory, HotUpdate};

use super::helper::{chapter_image, chapter_url_encode};

extern crate alloc;
use alloc::string::ToString;

const COVER_SERVER: &str = "https://temp.compsci88.com/cover/{{Result.i}}.jpg";

// Parse manga with title and cover
pub fn parse_manga_listing(manga_object: &HotUpdate) -> Result<Manga> {
	let id = &manga_object.id;
	let title = unescape_html_entities(&manga_object.title);
	let cover = String::from(COVER_SERVER).replace("{{Result.i}}", id);

	let mut url = defaults_get("sourceURL")?.as_string()?.read();
	url.push_str("/manga/");
	url.push_str(id);

	Ok(Manga {
		id: id.to_string(),
		title: title.to_string(),
		cover,
		url,
		..Default::default()
	})
}

pub fn parse_basic_manga(nepnep: &Directory) -> Result<Manga> {
	let id = &nepnep.id;
	let title = unescape_html_entities(&nepnep.title);
	let cover = String::from(COVER_SERVER).replace("{{Result.i}}", id);

	let mut url = defaults_get("sourceURL")?.as_string()?.read();
	url.push_str("/manga/");
	url.push_str(id);
	Ok(Manga {
		id: id.to_string(),
		title: title.to_string(),
		cover,
		url,
		..Default::default()
	})
}

// Parse complete manga info
pub fn parse_full_manga(id: String, url: String, manga_node: Node) -> Result<Manga> {
	let cover = manga_node
		.select("div.BoxBody > div.row img")
		.attr("src")
		.read();
	let title = manga_node.select("div.BoxBody > div.row h1").text().read();
	let author = manga_node
		.select("div.BoxBody > div.row li.list-group-item:has(span:contains(Author)) a")
		.first()
		.text()
		.read();
	let description = manga_node
		.select("div.BoxBody > div.row div.Content")
		.text()
		.read();

	let mut categories: Vec<String> = Vec::new();
	manga_node
		.select("li.list-group-item:has(span:contains(Genre)) a")
		.array()
		.for_each(|tag| categories.push(tag.as_node().expect("node array").text().read()));

	let status = match manga_node
		.select(
			"div.BoxBody > div.row li.list-group-item:has(span:contains(Status)) a:contains(Scan)",
		)
		.first()
		.text()
		.read()
		.as_str()
	{
		"Ongoing (Scan)" => MangaStatus::Ongoing,
		"Complete (Scan)" => MangaStatus::Completed,
		"Hiatus (Scan)" => MangaStatus::Hiatus,
		"Cancelled (Scan)" => MangaStatus::Cancelled,
		"Discontinued (Scan)" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	};

	let nsfw = if categories
		.iter()
		.any(|e| e == "Adult" || e == "Hentai" || e == "Mature")
	{
		MangaContentRating::Nsfw
	} else if categories.iter().any(|e| e == "Ecchi") {
		MangaContentRating::Suggestive
	} else {
		MangaContentRating::Safe
	};

	let viewer = match manga_node
		.select("li.list-group-item:has(span:contains(Type)) a")
		.first()
		.text()
		.read()
		.as_str()
	{
		"Manhua" => MangaViewer::Scroll,
		"Manhwa" => MangaViewer::Scroll,
		_ => MangaViewer::Rtl,
	};

	Ok(Manga {
		id,
		cover,
		title,
		author,
		description,
		url,
		categories,
		status,
		nsfw,
		viewer,
		..Default::default()
	})
}

// Parse chapter info
pub fn parse_chapter(manga_id: &str, chapter_object: ObjectRef) -> Result<Chapter> {
	let id = chapter_object.get("Chapter").as_string()?.read();

	let mut path = String::from(manga_id);
	path.push_str(&chapter_url_encode(&id));

	let chapter = id[1..].parse::<f32>().unwrap_or(-10.0) / 10.0;

	let mut title = match chapter_object.get("ChapterName").as_string() {
		Ok(title) => title.read(),
		Err(_) => String::new(),
	};
	if title.is_empty() {
		title = chapter_object.get("Type").as_string()?.read();
		title.push(' ');
		title.push_str(&chapter_image(&id, false));
	}

	let mut volume = -1.0;

	let cleaned_title = {
		let mut cleaned_title = title.split_whitespace().collect::<Vec<&str>>();

		// Remove leading season text and set volume accordingly
		// This is for titles like "S1 - Chapter 1"
		if title.len() >= 2 {
			let title_chars = cleaned_title[0].chars().collect::<Vec<char>>();

			if title_chars[0] == 'S' && title_chars[1].to_string().parse::<f64>().is_ok() {
				volume = title_chars[1].to_string().parse::<f32>().unwrap_or(-1.0);
				cleaned_title.remove(0);
			}

			// Remove leading symbols
			if !cleaned_title.is_empty() && cleaned_title[0] == "-" {
				cleaned_title.remove(0);
			}
		}

		// Remove leading volume text and set volume accordingly
		if cleaned_title.len() >= 2
			&& (cleaned_title[0] == "Volume")
			&& cleaned_title[1].parse::<f64>().is_ok()
		{
			volume = cleaned_title[1].parse::<f32>().unwrap_or(-1.0);
			cleaned_title.remove(0);
			cleaned_title.remove(0);
		}

		// Remove leading chapter text
		if cleaned_title.len() >= 2
			&& (cleaned_title[0] == "Chapter"
				|| cleaned_title[0] == "Episode"
				|| cleaned_title[0] == "episode."
				|| cleaned_title[0] == "No."
				|| cleaned_title[0] == "#")
			&& cleaned_title[1].parse::<f64>().is_ok()
		{
			cleaned_title.remove(0);
			cleaned_title.remove(0);
		}

		// Remove leading symbols
		if !cleaned_title.is_empty() && cleaned_title[0] == "-" {
			cleaned_title.remove(0);
		}

		cleaned_title.join(" ")
	};

	let date_updated = chapter_object
		.get("Date")
		.as_date("yyyy-MM-dd HH:mm:SS", Some("en-US"), Some("UTC"))
		.unwrap_or(-1.0);

	let mut url = String::from("https://mangasee123.com/read-online/");
	url.push_str(&path);

	Ok(Chapter {
		id: path,
		title: cleaned_title,
		volume,
		chapter,
		date_updated,
		url,
		lang: String::from("en"),
		..Default::default()
	})
}
