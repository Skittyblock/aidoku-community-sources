use aidoku::error::Result;
use aidoku::prelude::format;
use aidoku::std::ObjectRef;
use aidoku::{std::html::Node, Manga};
use aidoku::{Chapter, MangaContentRating, MangaPageResult, MangaViewer, Page};
use alloc::string::ToString;
use alloc::{string::String, vec::Vec};

use crate::helper::{get_date, get_image_src, manga_status, parse_chapter_number};

pub fn parse_manga_list(json: ObjectRef) -> Result<MangaPageResult> {
	let results = json.get("manga").as_array()?;
		let mut mangas: Vec<Manga> = Vec::new();
		for manga in results {
			if let Ok(manga_obj) = manga.as_object() {
				let title = match manga_obj.get("title").as_string() {
					Ok(node) => node.read(),
					Err(_) => continue,
				};
				let id = match manga_obj.get("url").as_string() {
					Ok(node) => node
						.read()
						.split('/')
						.nth_back(0)
						.unwrap_or_default()
						.to_string(),
					Err(_) => continue,
				};
				let cover = match manga_obj.get("image").as_string() {
					Ok(node) => node.read(),
					Err(_) => continue,
				};
				mangas.push(Manga {
					id,
					cover,
					title,
					..Default::default()
				});
			}
		}
		Ok(MangaPageResult {
			manga: mangas,
			has_more: false,
		})
}

pub fn parse_manga_listing(html: Node) -> Result<MangaPageResult> {
	let mut titles: Vec<Manga> = Vec::new();
	for manga in html.select(".main-slide figure").array() {
		let manga_node = manga.as_node().expect("node array");
		let title = String::from(manga_node.select("a").text().read().trim());
		let id = String::from(
			manga_node
				.select("a")
				.attr("href")
				.read()
				.split('/')
				.nth_back(0)
				.unwrap_or_default(),
		);
		let cover = get_image_src(&manga_node, "img");
		titles.push(Manga {
			id,
			cover,
			title,
			..Default::default()
		});
	}
	let last_page = html.select("li:nth-last-child(2) a.page-btn").text().read();
	let has_more = last_page.contains('»');
	Ok(MangaPageResult {
		manga: titles,
		has_more,
	})
}

pub fn parse_manga_details(id: String, html: Node) -> Result<Manga> {
	let title = html.select(".info-title").text().read().trim().to_string();
	let cover = get_image_src(&html, ".img-holder img");
	let author = html
		.select(".publisher.d-none > a:nth-child(1)")
		.first()
		.text()
		.read()
		.trim()
		.to_string();
	let description = html
		.select(".info-desc p")
		.text()
		.read()
		.split(':')
		.next_back()
		.unwrap_or("")
		.to_string();
	let status = manga_status(
		html.select(".info-detail  .source")
			.first()
			.text()
			.read()
			.to_uppercase(),
	);
	let url = html.select("#share_modal").attr("data-link").read();

	let mut categories = Vec::new();
	let mut nsfw = MangaContentRating::Safe;
	for node in html.select(".tags a").array() {
		let category = node.as_node().expect("node array").text().read();
		if [
			"+18",
			"ecchi",
			"erotic",
			"milf",
			"bdsm",
			"rape",
			"defloration",
			"lgbtq",
		]
		.contains(&category.to_lowercase().as_str())
		{
			nsfw = MangaContentRating::Nsfw;
		}
		categories.push(category);
	}
	let manga_type = html
		.select("table td:eq(0) div")
		.text()
		.read()
		.replace("Type", "")
		.replace(' ', "");
	let viewer = match manga_type.as_str() {
		"Japanese" => MangaViewer::Rtl,
		"Korean" => MangaViewer::Scroll,
		_ => MangaViewer::Rtl,
	};
	Ok(Manga {
		id: id.clone(),
		cover,
		title,
		author,
		description,
		url,
		categories,
		status,
		nsfw,
		viewer,
		..Default::default()
	})
}

pub fn parse_chapter_list(html: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	for chapter in html.select(".chapter-list li").array() {
		let node = chapter.as_node().expect("Failed to get node");
		let url = node.select("a").attr("href").to_string();
		let id = parse_chapter_number(&node.select("span:nth-child(1)").text().read());
		let date_updated = get_date(node.select("span:nth-child(2)").text().read());

		chapters.push(Chapter {
			id: id.to_string(),
			url,
			chapter: id,
			lang: String::from("en"),
			date_updated,
			..Default::default()
		})
	}

	Ok(chapters)
}

pub fn parse_page_list(html: Node, base_url: &str) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	for (index, page) in html.select("#imageContainer img").array().enumerate() {
		let page_node = page.as_node().expect("node array");
		let page_url = format!("{}/{}", base_url, page_node.attr("src").read());
		pages.push(Page {
			index: index.try_into().unwrap_or(-1),
			url: page_url,
			..Default::default()
		});
	}
	Ok(pages)
}
