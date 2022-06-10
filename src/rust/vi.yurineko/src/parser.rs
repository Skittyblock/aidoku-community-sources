use crate::helper::{extract_f32_from_string, i32_to_string, status_map, text_with_newlines};
use aidoku::{
	error::Result,
	prelude::format,
	std::{ObjectRef, String, Vec},
	Chapter, Manga, MangaContentRating, MangaViewer,
};

pub fn parse_manga(manga_object: ObjectRef) -> Result<Manga> {
	let id = manga_object.get("id").as_int().unwrap_or(-1);
	let title = manga_object.get("originalName").as_string()?.read();
	let cover = manga_object.get("thumbnail").as_string()?.read();
	let url = format!("https://yurineko.net/manga/{id}");

	let authors = manga_object.get("author").as_array()?;
	let author = authors
		.map(|author| {
			let author_object = author.as_object()?;
			Ok(author_object.get("name").as_string()?.read())
		})
		.map(|a: Result<String>| a.unwrap_or_default())
		.collect::<Vec<String>>()
		.join(", ");

	let description_html = manga_object.get("description").as_string()?.read();
	let description = text_with_newlines(description_html).unwrap_or_default();

	let tags = manga_object.get("tag").as_array()?;
	let couples = manga_object.get("couple").as_array()?;
	let categories = tags
		.chain(couples)
		.map(|tag| {
			let tag_object = tag.as_object()?;
			Ok(tag_object.get("name").as_string()?.read())
		})
		.map(|a: Result<String>| a.unwrap_or_default())
		.collect::<Vec<String>>();

	let mut nsfw: MangaContentRating = MangaContentRating::Safe;
	let mut viewer: MangaViewer = MangaViewer::Rtl;
	for tag in &categories {
		if tag.contains("sex") || tag.contains("NSFW") || tag.contains("R18") {
			nsfw = MangaContentRating::Nsfw;
		} else if tag.contains("Ecchi") {
			nsfw = MangaContentRating::Suggestive;
		}
		if tag.contains('>') || tag.contains("Manhua") || tag.contains("Manhwa") {
			viewer = MangaViewer::Ltr;
		}
	}
	Ok(Manga {
		id: i32_to_string(id as i32),
		cover,
		title,
		author,
		artist: String::new(),
		description,
		url,
		categories,
		status: status_map(manga_object.get("status").as_int().unwrap_or(-1)),
		nsfw,
		viewer,
	})
}

pub fn parse_chapter(scanlator: String, chapter_object: ObjectRef) -> Result<Chapter> {
	let id = chapter_object.get("id").as_int().unwrap_or(-1);
	let manga_id = chapter_object.get("mangaID").as_int().unwrap_or(-1);
	let chapter_id = format!("{manga_id}/{id}");
	let url = format!("https://yurineko.net/read/{chapter_id}");
	let title = chapter_object.get("name").as_string()?.read();
	let chapter_number = extract_f32_from_string(String::from("-"), String::from(&title));

	let date_string = chapter_object.get("date").as_string()?;
	let date_object = date_string
		.0
		.as_date("yyyy-MM-dd'T'HH:mm:ss.SSS'Z'", Some("en_US"), Some("UTC"))
		.unwrap_or(-1.0);

	Ok(Chapter {
		id: chapter_id,
		title,
		volume: -1.0,
		chapter: chapter_number,
		date_updated: date_object,
		scanlator,
		url,
		lang: String::from("vi"),
	})
}
