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
		id: String::from("multi.xkcd.ko"),
		cover: String::from(THUMBNAIL_URL),
		title: String::from("한국어로 xkcd"),
		author: String::from("랜들 먼로"),
		artist: String::from("랜들 먼로"),
		description: String::from("사랑, 풍자, 수학, 그리고 언어에 관한 웹 만화."),
		url: String::from("https://xkcdko.com"),
		categories: Vec::new(),
		status: MangaStatus::Ongoing,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Ltr,
	}
}

pub fn get_chapter_list() -> Result<Vec<Chapter>> {
	let html = Request::new("https://xkcdko.com/archive", HttpMethod::Get).html()?;
	Ok(html
		.select("#comicList > ol > li > a")
		.array()
		.filter_map(|elem| {
			elem.as_node()
				.map(|node| {
					let url = node.attr("abs:href").read();
					let chapter = extract_f32_from_string(node.attr("href").read())[0];
					Chapter {
						id: chapter.to_string(),
						title: node.text().read(),
						volume: -1.0,
						chapter,
						date_updated: -1.0,
						scanlator: String::new(),
						url,
						lang: String::from("ko"),
					}
				})
				.ok()
		})
		.collect::<Vec<_>>())
}

pub fn get_page_list(id: String) -> Result<Vec<Page>> {
	super::get_page_list(
		format!("https://xkcdko.com/{id}"),
		String::from("#comic img"),
		// Google translated, sorry
		true,
		format!(
			"이 만화의 대화형 버전을 경험하려면\n브라우저에서 엽니다. https://xkcdko.com/{id}/"
		),
		super::ImageVariant::Cjk,
	)
}
