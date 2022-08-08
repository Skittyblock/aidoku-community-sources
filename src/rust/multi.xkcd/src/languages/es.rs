use super::THUMBNAIL_URL;
use aidoku::{
	error::Result,
	prelude::format,
	std::{
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, Manga, MangaContentRating, MangaStatus, MangaViewer, Page,
};

pub fn comic_info() -> Manga {
	Manga {
		id: String::from("multi.xkcd.es"),
		cover: String::from(THUMBNAIL_URL),
		title: String::from("xkcd en español"),
		author: String::from("Randall Munroe"),
		artist: String::from("Randall Munroe"),
		description: String::from("Un webcómic sobre romance, sarcasmo, mates y lenguaje."),
		url: String::from("https://es.xkcd.com"),
		categories: Vec::new(),
		status: MangaStatus::Ongoing,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Ltr,
	}
}

pub fn get_chapter_list() -> Result<Vec<Chapter>> {
	let html = Request::new("https://es.xkcd.com/archive", HttpMethod::Get).html()?;
	let count = html.select("#archive-ul > ul > li > a").array().len();
	Ok(html
		.select("#archive-ul > ul > li > a")
		.array()
		.enumerate()
		.filter_map(|(idx, elem)| {
			elem.as_node()
				.map(|node| {
					let url = node.attr("abs:href").read();
					let id = node
						.attr("href")
						.read()
						.split('/')
						.filter_map(|val| {
							if val.is_empty() {
								None
							} else {
								Some(String::from(val))
							}
						})
						.last()
						.unwrap_or_default();
					Chapter {
						id,
						title: node.text().read(),
						volume: -1.0,
						chapter: (count as f32) - (idx as f32), /* Doesn't match the original,
						                                         * but what
						                                         * can I do /shrug */
						date_updated: -1.0,
						scanlator: String::new(),
						url,
						lang: String::from("es"),
					}
				})
				.ok()
		})
		.collect::<Vec<_>>())
}

pub fn get_page_list(id: String) -> Result<Vec<Page>> {
	super::get_page_list(
        format!("https://es.xkcd.com/strips/{id}/"),
        String::from("#middleContent .strip"),
        // DeepL translated, sorry.
        false,
        format!("Para experimentar la versión interactiva de este cómic,\nábralo en un navegador: https://es.xkcd.com/strips/{id}/"),
        super::ImageVariant::Latin,
    )
}
