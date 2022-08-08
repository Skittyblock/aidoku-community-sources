use super::{word_wrap, ToImageUrl, THUMBNAIL_URL};
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
use alloc::{string::ToString, vec};

pub fn comic_info() -> Manga {
	Manga {
		id: String::from("multi.xkcd.ru"),
		cover: String::from(THUMBNAIL_URL),
		title: String::from("xkcd на русском языке"),
		author: String::from("Рэндел Манро"),
		artist: String::from("Рэндел Манро"),
		description: String::from("о романтике, сарказме, математике и языке"),
		url: String::from("https://xkcd.ru"),
		categories: Vec::new(),
		status: MangaStatus::Ongoing,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Ltr,
	}
}

pub fn get_chapter_list() -> Result<Vec<Chapter>> {
	let html = Request::new("https://xkcd.ru/img", HttpMethod::Get).html()?;
	Ok(html
		.select(".main > a")
		.array()
		.filter_map(|elem| {
			elem.as_node()
				.map(|node| {
					let url = node.attr("abs:href").read();
					let chapter = extract_f32_from_string(node.attr("href").read())[0];
					let title = node.select("img").attr("alt").read();
					Chapter {
						id: chapter.to_string(),
						title,
						volume: -1.0,
						chapter,
						date_updated: -1.0,
						scanlator: String::new(),
						url,
						lang: String::from("ru"),
					}
				})
				.ok()
		})
		.collect::<Vec<_>>())
}

pub fn get_page_list(id: String) -> Result<Vec<Page>> {
	let html = Request::new(format!("https://xkcd.ru/{id}/"), HttpMethod::Get).html()?;
	let image_url = html.select(".main img[alt]").attr("abs:src").read();
	let title = html.select(".main img[alt]").attr("alt").read();
	let alt = html.select(".main .comics_text").text().read();
	Ok(vec![
		Page {
			index: 0,
			url: image_url,
			..Default::default()
		},
		Page {
			index: 1,
			url: word_wrap(title, alt).to_image_url(super::ImageVariant::Latin),
			..Default::default()
		},
	])
}
