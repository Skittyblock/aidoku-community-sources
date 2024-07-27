use aidoku::{
	helpers::substring::Substring,
	prelude::println,
	std::{html::Node, String, Vec},
	Chapter, Manga, MangaPageResult, Page,
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
