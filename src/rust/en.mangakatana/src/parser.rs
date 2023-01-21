use aidoku::{
	error::Result, prelude::format, std::html::Node, std::net::HttpMethod, std::net::Request,
	std::String, std::Vec, Chapter, Listing, Manga, MangaContentRating, MangaPageResult,
	MangaStatus, MangaViewer, Page,
};

use crate::helper::*;

pub fn parse_manga_list(html: Node, base_url: String) -> MangaPageResult {
	let mut manga: Vec<Manga> = Vec::new();

	for node in html.select("#book_list > .item").array() {
		let node = node.as_node().expect("Failed to get node");

		let raw_url = node.select(".text .title a").attr("href").read();
		let id = get_manga_id(raw_url);
		let url = get_manga_url(id.clone(), base_url.clone());
		let cover = node.select(".media .wrap_img img").attr("src").read();
		let title = node.select(".text .title a").text().read();
		let description = text_with_newlines(node.select(".text .summary"));

		let status = {
			let status_string = node.select(".media .status").text().read();
			get_manga_status(status_string)
		};

		let mut categories = Vec::new();
		for genre in node.select(".text .genres a").array() {
			let genre = genre.as_node().expect("Failed to get genre node");
			let genre = genre.text().read();
			categories.push(genre);
		}

		let nsfw = get_manga_content_rating(categories.clone());
		let viewer = get_manga_viewer(categories.clone());

		manga.push(Manga {
			id,
			cover,
			title,
			description,
			url,
			categories,
			status,
			nsfw,
			viewer,
			..Default::default()
		});
	}

	let has_more = !html.select(".uk-pagination .next").array().is_empty();

	MangaPageResult { manga, has_more }
}

pub fn parse_manga_details(html: Node, manga_url: String, base_url: String) -> Manga {
	let id = get_manga_id(manga_url);
	let cover = html.select("#single_book .cover img").attr("src").read();
	let title = html.select("#single_book .info .heading").text().read();
	let author = html
		.select("#single_book .info .meta .author")
		.text()
		.read();
	let description = text_with_newlines(html.select("#single_book .summary p"));

	let status = {
		let status_string = html
			.select("#single_book .info .meta .status")
			.text()
			.read();
		get_manga_status(status_string)
	};

	let mut categories = Vec::new();
	for genre in html.select("#single_book .info .meta .genres a").array() {
		let genre = genre.as_node().expect("Failed to get genre node");
		let genre = genre.text().read();
		categories.push(genre);
	}

	let nsfw = get_manga_content_rating(categories.clone());
	let viewer = get_manga_viewer(categories.clone());

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
