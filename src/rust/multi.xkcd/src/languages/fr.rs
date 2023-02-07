use super::{word_wrap, ToImageUrl, THUMBNAIL_URL};
use crate::helper::extract_f32_from_string;
use aidoku::{
	error::Result,
	prelude::*,
	std::{
		html::Node,
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, Manga, MangaContentRating, MangaStatus, MangaViewer, Page,
};
use alloc::vec;

pub fn comic_info() -> Manga {
	Manga {
		id: String::from("multi.xkcd.fr"),
		cover: String::from(THUMBNAIL_URL),
		title: String::from("xkcd en français"),
		author: String::from("Randall Munroe"),
		artist: String::from("Randall Munroe"),
		description: String::from(
			"Un webcomic sarcastique qui parle de romance, de maths et de langage.",
		),
		url: String::from("https://xkcd.lapin.org"),
		categories: Vec::new(),
		status: MangaStatus::Ongoing,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Ltr,
	}
}

pub fn get_chapter_list() -> Result<Vec<Chapter>> {
	let data = Request::new("https://xkcd.lapin.org/tous-episodes.php", HttpMethod::Get).data();
	let html = Node::new_with_uri(
		String::from_utf8_lossy(&data).as_ref(),
		"https://xkcd.lapin.org/tous-episodes.php",
	)?;
	Ok(html
		.select("#content a:not(:last-of-type)")
		.array()
		.rev()
		.filter_map(|elem| {
			elem.as_node()
				.map(|node| {
					let url = node.attr("abs:href").read();
					let chapter = String::from(&url[url.find('=').unwrap_or(0) + 1..]);
					Chapter {
						id: chapter.clone(),
						title: node.text().read(),
						volume: -1.0,
						chapter: extract_f32_from_string(chapter)[0],
						date_updated: -1.0,
						scanlator: String::new(),
						url,
						lang: String::from("fr"),
					}
				})
				.ok()
		})
		.collect::<Vec<_>>())
}

pub fn get_page_list(id: String) -> Result<Vec<Page>> {
	let data = Request::new(
		format!("https://xkcd.lapin.org/index.php?number={id}"),
		HttpMethod::Get,
	)
	.data();
	let html = Node::new_with_uri(
		String::from_utf8_lossy(&data).as_ref(),
		format!("https://xkcd.lapin.org/index.php?number={id}"),
	)?;
	let title = html
		.select("#col1 h2")
		.array()
		.get(0)
		.as_node()
		.map(|v| v.text().read())
		.unwrap_or_default();

	let image_node = html.select("#col1 img[title]");
	let image_url = image_node.attr("abs:src").read();
	let alt = image_node.attr("alt").read();
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
