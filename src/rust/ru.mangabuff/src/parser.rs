use aidoku::MangaStatus;
use aidoku::{
	helpers::uri::encode_uri,
	prelude::*,
	std::{current_date, String, StringRef, Vec},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaViewer, Page,
};

extern crate alloc;
use alloc::string::ToString;

use crate::{
	helpers::{get_base_url, get_manga_id, get_manga_thumb_url, get_manga_url, parse_status},
	wrappers::WNode,
};

pub fn parse_manga_list(html: &WNode) -> Option<Vec<Manga>> {
	let mut mangas = Vec::new();

	for card_node in html.select("div.cards") {
		let card_mangas = card_node
			.select("a.cards__item")
			.iter()
			.filter(|node| {
				node.attr("class")
					.is_none_or(|class| !class.contains("cloned"))
			})
			.filter_map(|manga_node| {
				let main_node = manga_node;

				let url = main_node.attr("href")?.to_string();
				let img_style = main_node
					.select_one("div.cards__img")?
					.attr("style")?
					.to_string();

				let id = get_manga_id(&url)?;
				let cover = get_manga_thumb_url(&img_style)?;
				let title_node = main_node.select_one("div.cards__name")?;

				Some(Manga {
					id,
					cover,
					title: title_node.text(),
					url,
					nsfw: MangaContentRating::default(),
					..Default::default()
				})
			})
			.collect::<Vec<_>>();

		mangas.extend(card_mangas);
	}

	if mangas.is_empty() {
		None
	} else {
		Some(mangas)
	}
}

pub fn parse_manga(html: &WNode, id: String) -> Option<Manga> {
	let main_node = html.select_one("div.manga")?;
	let description_node =
		html.select_one("div.tabs__content div.tabs__page[data-page=info] div.manga__description")?;
	if let Some(meta) = html.select_one("meta[property=og:image]") {
		if let Some(og_image) = meta.attr("content") {
			let cover = og_image;
			let url = get_manga_url(&id);
			let title = main_node
				.select_one("h1.manga__name")
				.map(|n| n.text().to_string())
				.or_else(|| {
					html.select_one("meta[property=og:title]")
						.and_then(|m| m.attr("content"))
				})
				.unwrap_or("".to_string())
				.to_string();
			let mut categories = html
				.select_one("div.tags")
				.map(|type_node| {
					type_node
						.select("a.tags__item")
						.iter()
						.map(WNode::text)
						.map(|s| s.trim().to_string())
						.collect::<Vec<_>>()
				})
				.unwrap_or_default();
			let mut mid_links = html.select("a.manga__middle-link");
			if mid_links.is_empty() {
				mid_links = main_node.select("div.manga__middle-links a");
			}
			let status = mid_links
				.iter()
				.find(|link| {
					!link.text().trim().is_empty()
						&& link
							.attr("href")
							.is_some_and(|href| href.to_string().contains("status_id"))
				})
				.map(|link| parse_status(link.text().trim()))
				.unwrap_or(MangaStatus::Unknown);
			let type_label = mid_links
				.iter()
				.find(|link| {
					!link.text().trim().is_empty()
						&& link
							.attr("href")
							.is_some_and(|href| href.to_string().contains("/types/"))
				})
				.map(|link| link.text().trim().to_string());
			if let Some(label) = &type_label {
				if !categories.iter().any(|c| c == label) {
					categories.push(label.clone());
				}
			}
			let viewer = match type_label.as_deref() {
				Some("Манхва") => MangaViewer::Scroll,
				Some("OEL-манга") => MangaViewer::Scroll,
				Some("Комикс Западный") => MangaViewer::Ltr,
				Some("Маньхуа") => MangaViewer::Scroll,
				Some("Манга") => MangaViewer::default(),
				_ => MangaViewer::default(),
			};
			let description = description_node.text().to_string();
			return Some(Manga {
				id,
				cover,
				title,
				author: "".to_string(),
				artist: "".to_string(),
				description,
				url,
				categories,
				status,
				nsfw: MangaContentRating::default(),
				viewer,
			});
		}
	}
	None
}

pub fn parse_chapters(html: &WNode, manga_id: &str) -> Option<Vec<Chapter>> {
	let chapter_nodes = html
		.select_one(
			"div.tabs__content div.tabs__page[data-page=chapters] div.chapters div.chapters__list",
		)
		.map(|list| list.select("a.chapters__item"))
		.unwrap_or_default();

	let mut chapters: Vec<_> = chapter_nodes
		.into_iter()
		.enumerate()
		.filter_map(|(idx, chapter_node)| {
			let url = chapter_node.attr("href")?.to_string();
			let id = url
				.trim_start_matches(&format!("{}/", get_manga_url(manga_id)))
				.trim_end_matches('/')
				.to_string();
			let title = chapter_node
				.select_one("div.chapters__name")
				.map(|name| {
					let t = name.text().trim().to_string();
					if t.is_empty() {
						chapter_node
							.select_one("div.chapters__value span")
							.map(|val| val.text().trim().to_string())
							.unwrap_or_else(|| format!("Глава {}", idx + 1))
					} else {
						t
					}
				})
				.unwrap_or_else(|| {
					chapter_node
						.select_one("div.chapters__value span")
						.map(|val| val.text().trim().to_string())
						.unwrap_or_else(|| format!("Глава {}", idx + 1))
				});
			let chapter = chapter_node
				.attr("data-chapter")
				.and_then(|ch| ch.parse::<f32>().ok())
				.unwrap_or_else(|| (idx + 1) as f32);
			let date_updated = chapter_node
				.attr("data-chapter-date")
				.map(|date_str| {
					let parsed = StringRef::from(date_str.trim()).as_date("dd.MM.yyyy", None, None);
					if parsed > 0.0 {
						parsed
					} else {
						current_date()
					}
				})
				.unwrap_or(current_date());
			Some(Chapter {
				id,
				title,
				chapter,
				date_updated,
				url,
				lang: "ru".to_string(),
				..Default::default()
			})
		})
		.collect();

	// Merge with chapters loaded via POST, deduplicating by id
	if let Some(mut extra) = parse_post_chapters(html, manga_id) {
		for ch in extra.drain(..) {
			if !chapters.iter().any(|c| c.id == ch.id) {
				chapters.push(ch);
			}
		}
	}

	Some(chapters)
}

pub fn parse_post_chapters(html: &WNode, manga_id: &str) -> Option<Vec<Chapter>> {
	let csrf_token = html
		.select_one("meta[name=csrf-token]")
		.and_then(|m| m.attr("content"))?;

	let data_id = html
		.select_one("div.manga")
		.and_then(|m| m.attr("data-id"))
		.unwrap_or(manga_id.to_string());

	let url = format!("{}/chapters/load", get_base_url());
	let body = format!("manga_id={}", data_id);
	let req = aidoku::std::net::Request::new(&url, aidoku::std::net::HttpMethod::Post)
		.header("X-CSRF-TOKEN", &csrf_token)
		.header("Content-Type", "application/x-www-form-urlencoded")
		.body(body.as_bytes());
	let resp_text = match req.string() {
		Ok(s) => s,
		Err(_) => return None,
	};

	let body_start = resp_text.find("<body>").map(|i| i + 6).unwrap_or(0);
	let body_end = resp_text.find("</body>").unwrap_or(resp_text.len());
	let mut inner = resp_text[body_start..body_end].to_string();
	inner = inner
		.replace("\\\"", "\"")
		.replace("\\/", "/")
		.replace("&quot;", "\"")
		.replace("&lt;", "<")
		.replace("&gt;", ">")
		.replace("&amp;", "&");
	let resp_node_fallback = WNode::_new(inner);
	let chapter_nodes = resp_node_fallback.select("a.chapters__item");

	let chapters = chapter_nodes
		.into_iter()
		.enumerate()
		.filter_map(|(idx, chapter_node)| {
			let url = chapter_node.attr("href")?.to_string();
			let id = url
				.trim_start_matches(&format!("{}/", get_manga_url(manga_id)))
				.trim_end_matches('/')
				.to_string();
			let title = chapter_node
				.select_one("div.chapters__name")
				.map(|name| {
					let t = name.text().trim().to_string();
					if t.is_empty() {
						chapter_node
							.select_one("div.chapters__value span")
							.map(|val| val.text().trim().to_string())
							.unwrap_or_else(|| format!("Глава {}", idx + 1))
					} else {
						t
					}
				})
				.unwrap_or_else(|| {
					chapter_node
						.select_one("div.chapters__value span")
						.map(|val| val.text().trim().to_string())
						.unwrap_or_else(|| format!("Глава {}", idx + 1))
				});
			let chapter = chapter_node
				.attr("data-chapter")
				.and_then(|ch| ch.parse::<f32>().ok())
				.unwrap_or_else(|| (idx + 1) as f32);
			let date_updated = chapter_node
				.attr("data-chapter-date")
				.map(|date_str| {
					let parsed = StringRef::from(date_str.trim()).as_date("dd.MM.yyyy", None, None);
					if parsed > 0.0 {
						parsed
					} else {
						current_date()
					}
				})
				.unwrap_or(current_date());
			Some(Chapter {
				id,
				title,
				chapter,
				date_updated,
				url,
				lang: "ru".to_string(),
				..Default::default()
			})
		})
		.collect::<Vec<_>>();

	if chapters.is_empty() {
		None
	} else {
		Some(chapters)
	}
}

pub fn get_page_list(html: &WNode) -> Option<Vec<Page>> {
	let reader_content_node = html.select_one("div.reader__pages")?;
	let item_nodes = reader_content_node.select("div.reader__item");
	let mut pages: Vec<(i32, String)> = item_nodes
		.into_iter()
		.filter_map(|item| {
			let page_num = item
				.attr("data-page")
				.and_then(|s| s.parse::<i32>().ok())
				.unwrap_or(0);
			let img = item.select_one("img")?;
			let url = img
				.attr("src")
				.or_else(|| img.attr("data-src"))?
				.trim()
				.to_string();
			Some((page_num, url))
		})
		.collect();
	pages.sort_by_key(|(n, _)| *n);
	let urls: Vec<String> = pages.into_iter().map(|(_, u)| u).collect();

	Some(
		urls.into_iter()
			.enumerate()
			.map(|(idx, url)| Page {
				index: idx as i32,
				url,
				..Default::default()
			})
			.collect(),
	)
}

pub fn get_filter_url(filters: &[Filter], page: i32) -> Option<String> {
	const QUERY_PART: &str = "&q=";

	let filter_addition: String = filters
		.iter()
		.filter_map(|filter| match filter.kind {
			FilterType::Title => {
				let value = filter.value.clone().as_string().ok()?.read();
				Some(format!("{QUERY_PART}{}", encode_uri(value)))
			}
			_ => None,
		})
		.collect();

	let filter_addition = match filter_addition.find(QUERY_PART) {
		Some(_) => filter_addition,
		None => filter_addition + QUERY_PART,
	};

	Some(format!(
		"{}/search?type=manga&page={}{}",
		get_base_url(),
		page,
		filter_addition
	))
}
