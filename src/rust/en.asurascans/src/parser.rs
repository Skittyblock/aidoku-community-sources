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
