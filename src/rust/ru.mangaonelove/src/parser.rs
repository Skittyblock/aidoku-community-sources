use core::iter::once;

use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	helpers::{substring::Substring, uri::encode_uri},
	prelude::*,
	std::{String, StringRef, Vec},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaContentRating, MangaStatus,
	MangaViewer, Page,
};

extern crate alloc;
use alloc::string::ToString;

use itertools::chain;

use crate::{
	constants::{BASE_URL, BASE_URL_READMANGA, PAGE_DIR},
	get_manga_details,
	helpers::{self, get_manga_id},
	wrappers::WNode,
};

pub fn parse_lising(html: &WNode, listing: Listing) -> Result<Vec<Manga>> {
	let sidebar_class = match listing.name.as_str() {
		"Популярное" => "c-top-sidebar",
		"Новое" => "c-top-second-sidebar",
		_ => return Err(WNode::PARSING_ERROR),
	};

	let sidebar_node = html
		.select_one(&format!("div.c-sidebar.{sidebar_class}"))
		.ok_or(WNode::PARSING_ERROR)?;

	let mangas = sidebar_node
		.select("div.slider__item")
		.iter()
		.filter_map(|manga_node| {
			let thumb_node = manga_node.select_one("div.slider__thumb_item")?;
			let desc_node = manga_node.select_one("div.slider__content_item")?;

			let thumb_link_node = thumb_node.select_one("a")?;
			let url = thumb_link_node.attr("href")?;
			let id = get_manga_id(&url)?;

			let cover = thumb_link_node.select_one("img")?.attr("src")?;

			let title = desc_node.select_one("div.post-title")?.text();

			let nsfw = match thumb_node.select_one("span") {
				Some(span_node) => {
					if span_node.text().contains("18+") {
						MangaContentRating::Nsfw
					} else {
						MangaContentRating::Suggestive
					}
				}
				None => MangaContentRating::Safe,
			};

			Some(Manga {
				id,
				cover,
				title,
				url,
				nsfw,
				..Default::default()
			})
		})
		.collect();

	Ok(mangas)
}

pub fn parse_search_results(html: &WNode) -> Result<Vec<Manga>> {
	let list_node = html
		.select_one("div.c-page-content div.main-col-inner div.tab-content-wrap div.c-tabs-item")
		.ok_or(WNode::PARSING_ERROR)?;

	let mangas = list_node
		.select("div.row.c-tabs-item__content")
		.into_iter()
		.filter_map(|manga_node| {
			let thumb_node = manga_node.select_one("div.tab-thumb")?;
			let summary_node = manga_node.select_one("div.tab-summary")?;

			let title_node = summary_node.select_one("div.post-title a")?;
			let content_node = summary_node.select_one("div.post-content")?;

			let extract_from_content = |class_name| {
				content_node
					.select_one(&format!("div.{class_name}"))?
					.select_one("div.summary-content")
					.map(|n| n.text())
			};

			let url = title_node.attr("href")?;
			let id = get_manga_id(&url)?;
			let cover = thumb_node.select_one("img")?.attr("src")?;
			let title = title_node.text();
			let author = extract_from_content("mg_author").unwrap_or_default();
			let artist = extract_from_content("mg_artists").unwrap_or_default();
			let categories: Vec<String> = content_node
				.select("div.mg_genres a")
				.iter()
				.map(WNode::text)
				.collect();
			let status = match extract_from_content("mg_status")?.trim() {
				"Онгоинг" => MangaStatus::Ongoing,
				"Завершен" => MangaStatus::Completed,
				"Брошено" => MangaStatus::Cancelled,
				"Заморожен" => MangaStatus::Hiatus,
				_ => MangaStatus::Unknown,
			};
			let nsfw = match categories.iter().find(|c| c.contains("18+")) {
				Some(_) => MangaContentRating::Nsfw,
				None => MangaContentRating::Suggestive,
			};

			Some(Manga {
				id,
				cover,
				title,
				author,
				artist,
				url,
				categories,
				status,
				nsfw,
				..Default::default()
			})
		})
		.collect();

	Ok(mangas)
}

fn get_manga_page_main_node(html: &WNode) -> Result<WNode> {
	html.select("div.leftContent")
		.pop()
		.ok_or(WNode::PARSING_ERROR)
}

pub fn parse_manga(html: &WNode, id: String) -> Result<Manga> {
	let main_node = get_manga_page_main_node(html)?;

	let main_attributes_node = main_node
		.select("div.flex-row")
		.pop()
		.ok_or(WNode::PARSING_ERROR)?;

	let picture_fororama_node = main_attributes_node.select("div.picture-fotorama").pop();
	let cover = picture_fororama_node
		.and_then(|pfn| pfn.select("img").pop())
		.and_then(|img_node| img_node.attr("src"))
		.unwrap_or_default();

	let names_node = main_node
		.select("h1.names")
		.pop()
		.ok_or(WNode::PARSING_ERROR)?;
	let title = names_node
		.select("span")
		.into_iter()
		.map(|name_node| name_node.text())
		.intersperse(" | ".to_string())
		.collect();

	let main_info_node = main_attributes_node
		.select("div.subject-meta")
		.pop()
		.ok_or(WNode::PARSING_ERROR)?;

	let extract_info_iter = |elem_class, link_type| {
		main_info_node
			.select(&format!("span.elem_{elem_class}"))
			.into_iter()
			.flat_map(move |node| {
				node.select(&format!("a.{link_type}-link"))
					.into_iter()
					.map(|person_node| person_node.text())
			})
	};

	let author = chain!(
		extract_info_iter("author", "person"),
		extract_info_iter("screenwriter", "person")
	)
	.intersperse(", ".to_string())
	.collect();

	let artist = extract_info_iter("illustrator", "person")
		.intersperse(", ".to_string())
		.collect();

	let description = main_node
		.select("meta")
		.into_iter()
		.find(|mn| {
			if let Some(itemprop) = mn.attr("itemprop") {
				return itemprop == "description";
			}
			false
		})
		.and_then(|desc_node| desc_node.attr("content"))
		.unwrap_or_default();

	let url = helpers::get_manga_url_readmanga(&id);

	let category_opt = extract_info_iter("category", "element").next();

	let viewer = match &category_opt {
		Some(category) => match category.to_lowercase().as_str() {
			"oel-манга" => MangaViewer::Scroll,
			"комикс" => MangaViewer::Ltr,
			"манхва" => MangaViewer::Scroll,
			"маньхуа" => MangaViewer::Scroll,
			_ => MangaViewer::default(),
		},
		None => MangaViewer::default(),
	};

	let categories = chain!(
		once(category_opt).flatten(),
		extract_info_iter("genre", "element")
	)
	.collect();

	let status_str_opt = main_info_node
		.select("p")
		.into_iter()
		.filter(|pn| pn.attr("class").is_none())
		.flat_map(|pn| pn.select("span"))
		.find(|sn| {
			if let Some(class_attr) = sn.attr("class") {
				return class_attr
					.split_whitespace()
					.any(|cl| cl.starts_with("text-"));
			}
			false
		})
		.map(|status_node| status_node.text());
	let status = match status_str_opt {
		Some(status_str) => match status_str.to_lowercase().as_str() {
			"переведено" => MangaStatus::Completed,
			"продолжается" => MangaStatus::Ongoing,
			"приостановлен" => MangaStatus::Hiatus,
			_ => MangaStatus::Unknown,
		},
		None => MangaStatus::Unknown,
	};

	Ok(Manga {
		id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories,
		status,
		nsfw: MangaContentRating::default(),
		viewer,
	})
}

pub fn parse_chapters_readmanga(html: &WNode, manga_id: &str) -> Result<Vec<Chapter>> {
	let main_node = get_manga_page_main_node(html)?;

	let chapters = main_node
		.select(
			"div[class~=chapters] > table > tbody > tr:has(td > a):has(td.date:not(.text-info))",
		)
		.into_iter()
		.filter_map(|chapter_elem| {
			let link_elem = chapter_elem.select("a.chapter-link").pop()?;

			// this: `chapter_elem.select("td.d-none")` doesn't work here, I don't know why
			let date_elems: Vec<_> = {
				let chapter_repr = chapter_elem.to_str();

				chapter_repr
					.match_indices("<td")
					.zip(chapter_repr.match_indices("</td>"))
					.map(|((start, _), (end, td_end))| {
						chapter_repr[start..end + td_end.len()].to_string()
					})
					.filter_map(|td_repr| WNode::new(td_repr).attr("data-date-raw"))
					.collect()
			};

			let chapter_rel_url = link_elem.attr("href")?;

			let id = chapter_rel_url
				.strip_prefix(format!("/{manga_id}/").as_str())?
				.to_string();

			let full_title = link_elem.text().replace(" новое", "").trim().to_string();
			let title = {
				let strippred_title: String = full_title
					.chars()
					.skip_while(|char| char.is_numeric() || char.is_whitespace() || char == &'-')
					.collect();
				if strippred_title.is_empty() {
					full_title
				} else {
					strippred_title
				}
			};

			let (vol_str, chap_str) = id.split_once('/')?;
			let volume = vol_str.strip_prefix("vol")?.parse().ok()?;
			let chapter = chap_str.parse().ok()?;

			let date_updated = {
				match date_elems.first() {
					Some(date_updated_str) => StringRef::from(&date_updated_str).as_date(
						"yyyy-MM-dd HH:mm:ss.SSS",
						None,
						None,
					),
					None => 0f64,
				}
			};

			let scanlator = link_elem
				.attr("title")
				.unwrap_or_default()
				.replace(" (Переводчик)", "");

			let url = helpers::get_chapter_url_readmanga(manga_id, &id);

			Some(Chapter {
				id,
				title,
				volume,
				chapter,
				date_updated,
				scanlator,
				url,
				lang: "ru".to_string(),
			})
		})
		.collect();

	Ok(chapters)
}

pub fn get_page_list_readmanga(html: &WNode) -> Result<Vec<Page>> {
	let script_text = html
		.select(r"div.reader-controller > script[type=text/javascript]")
		.pop()
		.map(|script_node| script_node.data())
		.ok_or(WNode::PARSING_ERROR)?;

	let chapters_list_str = script_text
		.find("[[")
		.zip(script_text.find("]]"))
		.map(|(start, end)| &script_text[start..end + 2])
		.ok_or(WNode::PARSING_ERROR)?;

	let urls: Vec<_> = chapters_list_str
		.match_indices("['")
		// extracting parts from ['https://t1.rmr.rocks/', '', "auto/68/88/46/0098.png_res.jpg", 959, 1400] into tuples
		.zip(chapters_list_str.match_indices("\","))
		.filter_map(|((l, _), (r, _))| {
			use itertools::Itertools;
			chapters_list_str[l + 1..r + 1]
				.replace(['\'', '"'], "")
				.split(',')
				.map(ToString::to_string)
				.collect_tuple()
		})
		// composing URL
		.map(|(part0, part1, part2)| {
			if part1.is_empty() && part2.starts_with("/static/") {
				format!("{BASE_URL_READMANGA}{part2}")
			} else if part1.starts_with("/manga/") {
				format!("{part0}{part2}")
			} else {
				format!("{part0}{part1}{part2}")
			}
		})
		// fixing URL
		.map(|url| {
			if !url.contains("://") {
				format!("https:{url}")
			} else {
				url
			}
		})
		.filter_map(|url| {
			if url.contains("one-way.work") {
				url.substring_before("?").map(ToString::to_string)
			} else {
				Some(url)
			}
		})
		.collect();

	Ok(urls
		.into_iter()
		.enumerate()
		.map(|(idx, url)| Page {
			index: idx as i32,
			url,
			..Default::default()
		})
		.collect())
}

pub fn get_filter_url(filters: &[Filter], page: i32) -> Result<String> {
	const QUERY_PART: &str = "&s=";

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

	Ok(format!(
		"{BASE_URL}/{PAGE_DIR}/{page}/?post_type=wp-manga&m_orderby=trending{}",
		filter_addition
	))
}

pub fn parse_incoming_url_readmanga(url: &str) -> Result<DeepLink> {
	let manga_id = match url.find("://") {
		Some(idx) => &url[idx + 3..],
		None => url,
	}
	.split('/')
	.next()
	.ok_or(AidokuError {
		reason: AidokuErrorKind::Unimplemented,
	})?;

	Ok(DeepLink {
		manga: Some(get_manga_details(manga_id.to_string())?),
		chapter: None,
	})
}
