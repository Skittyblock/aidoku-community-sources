use super::THUMBNAIL_URL;
use crate::helper::extract_f32_from_string;
use aidoku::{
	error::Result,
	prelude::format,
	std::{
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, Manga, MangaContentRating, MangaStatus, MangaViewer, Page,
};
use alloc::string::ToString;

pub fn comic_info() -> Manga {
	Manga {
		id: String::from("multi.xkcd.en"),
		cover: String::from(THUMBNAIL_URL),
		title: String::from("xkcd"),
		author: String::from("Randall Munroe"),
		artist: String::from("Randall Munroe"),
		description: String::from("A webcomic of romance, sarcasm, math and language."),
		url: String::from("https://xkcd.com"),
		categories: Vec::new(),
		status: MangaStatus::Ongoing,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Ltr,
	}
}

pub fn get_chapter_list() -> Result<Vec<Chapter>> {
	let html = Request::new("https://xkcd.com/archive", HttpMethod::Get).html()?;
	Ok(html
		.select("#middleContainer > a")
		.array()
		.filter_map(|elem| {
			elem.as_node()
				.map(|node| {
					let url = node.attr("abs:href").read();
					let date_updated = node
						.attr("title")
						.0
						.as_date("yyyy-M-d", None, None)
						.unwrap_or(-1.0);
					let chapter = extract_f32_from_string(node.attr("href").read())[0];
					Chapter {
						id: chapter.to_string(),
						title: node.text().read(),
						volume: -1.0,
						chapter,
						date_updated,
						scanlator: String::new(),
						url,
						lang: String::from("en"),
					}
				})
				.ok()
		})
		.collect::<Vec<_>>())
}

pub fn get_page_list(id: String) -> Result<Vec<Page>> {
	super::get_page_list(
        format!("https://xkcd.com/{id}"),
        String::from("#comic img"),
        false,
        format!("To experience the interactive version of this comic,\nopen it in a browser: https://xkcd.com/{id}/"),
        super::ImageVariant::Latin,
    )
}
