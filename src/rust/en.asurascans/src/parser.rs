use aidoku::{
	helpers::substring::Substring,
	prelude::println,
	std::{html::Node, String, Vec},
	Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page,
};

use crate::helper::*;

pub fn parse_manga_list(html: Node) -> MangaPageResult {
	let mut manga: Vec<Manga> = Vec::new();

	for node in html.select("div.grid > a[href]").array() {
		if let Ok(node) = node.as_node() {
			let raw_url = node.attr("href").read();

			let id = get_manga_id(&raw_url).expect("Failed to get manga id");
			let url = get_manga_url(&id);
			let cover = node.select("img").attr("src").read();
			let title = node.select("div.block > span.block").text().read();

			manga.push(Manga {
				id,
				cover,
				title,
				url,
				..Default::default()
			});
		}
	}

	let has_more = !html
		.select("div.flex > a.flex.bg-themecolor:contains(Next)")
		.array()
		.is_empty();

	MangaPageResult { manga, has_more }
}

pub fn parse_manga_details(html: Node, manga_id: String) -> Manga {
	let url = get_manga_url(&manga_id);

	let wrapper = html.select("div.relative.grid");
	let cover = wrapper.select("img[alt=poster]").attr("src").read();
	let title = wrapper.select("span.text-xl.font-bold").text().read();
	let author = wrapper
		.select("div:has(h3:eq(0):containsOwn(Author)) > h3:eq(1)")
		.text()
		.read();
	let artist = wrapper
		.select("div:has(h3:eq(0):containsOwn(Artist)) > h3:eq(1)")
		.text()
		.read();
	let description = wrapper.select("span.font-medium.text-sm").text().read();

	let mut categories = Vec::new();

	let mut nsfw = MangaContentRating::Safe;

	for genre in wrapper
		.select("div[class^=space] > div.flex > button.text-white")
		.array()
	{
		let genre = genre.as_node().expect("Failed to get genre node");
		let genre = genre.text().read();

		if genre == "Adult" || genre == "Ecchi" {
			nsfw = MangaContentRating::Suggestive;
		}

		categories.push(genre);
	}

	let status = {
		let status_string = wrapper
			.select("div.flex:has(h3:eq(0):containsOwn(Status)) > h3:eq(1)")
			.text()
			.read();

		match status_string.as_str() {
			"Ongoing" => MangaStatus::Ongoing,
			"Hiatus" => MangaStatus::Hiatus,
			"Completed" => MangaStatus::Completed,
			"Dropped" => MangaStatus::Cancelled,
			_ => MangaStatus::Unknown,
		}
	};

	let viewer = {
		let format = wrapper
			.select("div.flex:has(h3:eq(0):containsOwn(Type)) > h3:eq(1)")
			.text()
			.read();

		match format.as_str() {
			"Manhwa" => MangaViewer::Scroll,
			"Manhua" => MangaViewer::Scroll,
			"Manga" => MangaViewer::Rtl,
			_ => MangaViewer::Scroll,
		}
	};

	Manga {
		id: manga_id,
		cover,
		title,
		author,
		artist,
		description,
		url,
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
		.select("div.scrollbar-thumb-themecolor > div.group")
		.array()
	{
		let node = node.as_node().expect("Failed to get chapter node");

		let raw_url = node.select("a").attr("abs:href").read();

		let id = get_chapter_id(&raw_url).expect("Failed to get chapter id");
		let manga_id = get_manga_id(&raw_url).expect("Failed to get manga id");

		let url = get_chapter_url(&id, &manga_id);

		// Chapter's title if it exists
		let title = node.select("h3:eq(0) > a > span").text().read();

		let chapter = node
			.select("h3:eq(0) > a")
			.text()
			.read()
			.replace(&title, "")
			.replace("Chapter", "")
			.trim()
			.parse::<f32>()
			.unwrap_or(-1.0);

		let date_updated =
			node.select("h3:eq(1)")
				.text()
				.as_date("MMMM d yyyy", Some("en-US"), None);

		chapters.push(Chapter {
			id,
			title,
			chapter,
			date_updated,
			url,
			..Default::default()
		});
	}

	chapters
}

pub fn parse_page_list(html: Node) -> Vec<Page> {
	let mut pages: Vec<Page> = Vec::new();

	for node in html.select("div > img[alt=chapter]").array() {
		let node = node.as_node().expect("Failed to get page node");

		let url = node.attr("src").read();
		let index = {
			let before = url.substring_after_last('/').unwrap_or("");
			let after = before.substring_before('.').unwrap_or("");
			after.parse::<i32>().unwrap_or(-1)
		};

		pages.push(Page {
			index,
			url,
			..Default::default()
		});
	}

	pages
}
