use aidoku::{
	prelude::format,
	std::{html::Node, String, Vec},
	Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page,
};

use crate::helper::*;
use crate::BASE_URL;

pub fn parse_manga_list(html: Node, searching: bool) -> MangaPageResult {
	let mut manga: Vec<Manga> = Vec::new();

	if searching {
		for node in html.select("a").array() {
			let node = node.as_node().expect("Failed to get node");

			let raw_url = node.attr("href").read();
			let id = get_manga_id(&raw_url);
			let url = get_manga_url(&id);
			let cover = format!("{}/img/noimg.jpg", BASE_URL);
			let title = String::from(node.text().read().trim());

			manga.push(Manga {
				id,
				cover,
				title,
				url,
				..Default::default()
			})
		}

		return MangaPageResult {
			manga,
			has_more: false,
		};
	}

	for node in html.select("#content .updates-item .leftside a").array() {
		let node = node.as_node().expect("Failed to get node");

		let raw_url = node.attr("href").read();

		let title = node.attr("title").read();
		let id = get_manga_id(&raw_url);
		let url = get_manga_url(&id);
		let cover = node.select("img").attr("src").read();

		manga.push(Manga {
			id,
			cover,
			title,
			url,
			..Default::default()
		});
	}

	let has_more = !manga.is_empty();

	MangaPageResult { manga, has_more }
}

pub fn parse_manga_details(html: Node, manga_url: String) -> Manga {
	let id = get_manga_id(&manga_url);
	let title = html.select("#novel .novel-title").text().read();

	let wrapper = html.select("#novel .novel-header");

	// yes there is a typo on the website
	let cover = wrapper.select("#thumbonail").attr("src").read();

	let author = String::from(
		wrapper
			.select("#mangainfo .author")
			.text()
			.read()
			.replace("Author:", "")
			.replace("updating", "")
			.trim(),
	);

	let description = {
		let description = text_with_newlines(wrapper.select("#info .description"));

		if description == "Not Provided" {
			String::new()
		} else {
			description
		}
	};

	let status = {
		let status_string = wrapper.select("#mangainfo .header-stats").text().read();
		if status_string.contains("Ongoing") {
			MangaStatus::Ongoing
		} else if status_string.contains("Completed") {
			MangaStatus::Completed
		} else {
			MangaStatus::Unknown
		}
	};

	let mut categories = Vec::new();
	for genre in wrapper.select("#mangainfo .categories ul li a").array() {
		let genre = genre.as_node().expect("Failed to get genre node");
		let genre = String::from(genre.text().read().trim());
		categories.push(genre);
	}

	let nsfw = {
		let mut rating = MangaContentRating::Safe;

		if !categories.is_empty() {
			if categories
				.iter()
				.any(|e| e == "Ecchi" || e == "Harem" || e == "Mature")
			{
				rating = MangaContentRating::Suggestive;
			}
			if categories.iter().any(|e| e == "Smut") {
				rating = MangaContentRating::Nsfw;
			}
		}

		rating
	};

	let viewer = MangaViewer::Scroll;

	Manga {
		id,
		cover,
		title,
		author,
		description,
		url: manga_url,
		categories,
		status,
		nsfw,
		viewer,
		..Default::default()
	}
}

pub fn parse_chapter_list(html: Node) -> Vec<Chapter> {
	let mut chapters: Vec<Chapter> = Vec::new();

	for node in html
		.select("#novel .novel-header #mangainfo #chapters .chapter-list li")
		.array()
	{
		let node = node.as_node().expect("Failed to get chapter node");

		let raw_url = node.select("a").attr("href").read();
		let id = node.attr("data-chapterno").read();

		let manga_id = get_manga_id(&raw_url);
		let url = get_chapter_url(&id, &manga_id);

		let chapter = node
			.attr("data-chapterno")
			.read()
			.parse::<f32>()
			.unwrap_or(-1.0);

		let volume = node
			.attr("data-volumeno")
			.read()
			.parse::<f32>()
			.unwrap_or(-1.0);

		let date_updated =
			node.select(".chapter-update")
				.attr("date")
				.as_date("yyyy-MM-dd", Some("en-US"), None);

		chapters.push(Chapter {
			id,
			chapter,
			volume: if volume == 0.0 { -1.0 } else { volume },
			date_updated,
			url,
			..Default::default()
		});
	}

	chapters
}

pub fn parse_page_list(html: Node) -> Vec<Page> {
	let mut pages: Vec<Page> = Vec::new();

	for node in html.select("main .wrapper center img.imgholder").array() {
		let node = node.as_node().expect("Failed to get image node");

		let url = node.attr("src").read();

		let index = url
			.split('/')
			.last()
			.and_then(|part| part.split('.').next())
			.and_then(|part| part.parse::<i32>().ok())
			.unwrap_or(-1);

		pages.push(Page {
			index,
			url,
			..Default::default()
		});
	}

	pages
}
