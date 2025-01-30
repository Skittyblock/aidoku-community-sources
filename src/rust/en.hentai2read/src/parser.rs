use aidoku::error::Result;
use aidoku::prelude::format;
use aidoku::std::json;
use aidoku::{std::html::Node, Manga};
use aidoku::{Chapter, MangaContentRating, MangaStatus, MangaViewer, Page};
use alloc::string::ToString;
use alloc::{string::String, vec::Vec};

use crate::helper::{
	between_string, clean_cover_url, get_manga_id, parse_chapter_number, BASE_URL,
};

pub fn parse_search(html: &Node) -> Vec<Manga> {
	let mut manga_arr: Vec<Manga> = Vec::new();
	for result in html.select(".book-grid-item").array() {
		let result_node = result.as_node().expect("Failed to get result node");
		let cover_url = result_node.select("img").first().attr("abs:src").read();
		let cover = clean_cover_url(&cover_url);

		let manga_url_node = result_node.select("a");
		let manga_url = manga_url_node.attr("href").to_string();
		let manga_id = get_manga_id(&manga_url);
		let title = manga_url_node.select("span.title-text").text().to_string();

		manga_arr.push(Manga {
			id: manga_id,
			cover,
			title,
			url: manga_url,
			status: MangaStatus::Completed,
			nsfw: MangaContentRating::Nsfw,
			viewer: MangaViewer::Rtl,
			..Default::default()
		});
	}

	manga_arr
}

pub fn parse_manga(id: String, html: Node) -> Result<Manga> {
	let mut author = String::new();
	let mut artist = String::new();
	let mut categories = Vec::new();
	let mut status = MangaStatus::Unknown;

	let title_node = html.select(".content .block-header h3.block-title a");
	let title = title_node.own_text().read();

	let cover = html.select(".img-container a img").attr("src").to_string();

	let url = format!("{BASE_URL}/{id}/");

	for item in html
		.select(".list.list-simple-mini li.text-primary")
		.array()
	{
		let li = item.as_node().expect("Failed to get li node");
		let key = li.select("b").text().read();
		match key.to_lowercase().as_str() {
			"status" => {
				let tag_button = li.select("a.tagButton").text().read().to_lowercase();
				status = match tag_button.as_str() {
					"ongoing" => MangaStatus::Ongoing,
					"completed" => MangaStatus::Completed,
					_ => MangaStatus::Unknown,
				}
			}
			"author" => {
				author = li.select("a.tagButton").text().read();
			}
			"artist" => {
				artist = li.select("a.tagButton").text().read();
			}
			"content" | "categories" => {
				for category in li.select("a.tagButton").array() {
					let category = category.as_node().expect("Failed to get category node");
					categories.push(category.own_text().read());
				}
			}

			_ => continue,
		}
	}

	Ok(Manga {
		id,
		title,
		author,
		artist,
		cover,
		url,
		categories,
		status,
		nsfw: MangaContentRating::Nsfw,
		viewer: MangaViewer::Rtl,
		..Default::default()
	})
}

pub fn parse_chapter_list(html: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	for link in html.select("ul.nav-chapters > li > div.media > a").array() {
		let node = link.as_node().expect("Failed to get node");
		let title = node.own_text().read();
		let url = node.attr("href").to_string();
		let chapter = parse_chapter_number(&url);

		chapters.push(Chapter {
			id: chapter.to_string(),
			title,
			chapter,
			url,
			lang: String::from("en"),
			..Default::default()
		})
	}

	Ok(chapters)
}

pub fn parse_page_list(html: Node) -> Result<Vec<Page>> {
	let mut script = String::new();
	for el in html.select("script").array() {
		let node = el.as_node().expect("script tag not found in html");
		let text = node.html().read();
		if text.contains("gData") {
			script = text;
			break;
		}
	}

	let Some(obj) = between_string(&script, "'images' : [", "\"]") else {
		return Ok(Vec::new());
	};

	let arr_str = format!("[{}\"]", obj);
	let arr = json::parse(arr_str)?.as_array()?;

	let mut pages: Vec<Page> = Vec::new();

	for (i, item) in arr.enumerate() {
		let img_path = item.as_string().unwrap_or_default();
		let img_url = format!("https://static.hentaicdn.com/hentai/{}", img_path);
		pages.push(Page {
			index: i as i32,
			url: img_url.to_string(),
			..Default::default()
		})
	}

	Ok(pages)
}
