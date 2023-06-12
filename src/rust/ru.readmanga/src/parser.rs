use core::iter::once;

use aidoku::{
	error::{AidokuError, AidokuErrorKind, NodeError, Result},
	helpers::{substring::Substring, uri::encode_uri},
	prelude::*,
	std::{
		net::{HttpMethod, Request},
		String, StringRef, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

extern crate alloc;
use alloc::{boxed::Box, string::ToString};

use itertools::chain;

use crate::{
	constants::{BASE_SEARCH_URL, BASE_URL, SEARCH_OFFSET_STEP},
	get_manga_details,
	sorting::Sorting,
	wrappers::{debug, WNode},
};

pub fn get_html(url: &str) -> Result<WNode> {
	Request::new(url, HttpMethod::Get)
		.html()
		.map(WNode::from_node)
}

pub fn get_manga_url(id: &str) -> String {
	format!("{}/{}", BASE_URL, id)
}

pub fn create_manga_page_result(mangas: Vec<Manga>) -> MangaPageResult {
	let has_more = mangas.len() == SEARCH_OFFSET_STEP as usize;
	MangaPageResult {
		manga: mangas,
		has_more,
	}
}

pub fn parse_search_results(html: &WNode) -> Result<Vec<Manga>> {
	let nodes = html.select("div.tile");
	// debug!("{:?}", nodes);

	let mangas: Vec<_> = nodes
		.into_iter()
		.filter_map(|node| {
			let div_img_node = node.select("div.img").pop()?;
			// debug!("div_img_node: {div_img_node:?}");

			let id = {
				let a_non_hover_node = div_img_node.select("a.non-hover").pop()?;
				// debug!("a_non_hover_node: {a_non_hover_node:?}");
				a_non_hover_node
					.attr("href")?
					.trim_start_matches('/')
					.to_string()
			};
			// debug!("id: {id}");

			let img_node = div_img_node.select("img").pop()?;
			// debug!("img_node: {img_node:?}");
			let cover = img_node.attr("original")?;
			// debug!("cover: {cover}");

			let title = img_node.attr("title")?;
			// debug!("title: {title}");

			let div_desc_node = node.select("div.desc").pop()?;

			let div_tile_info_node = div_desc_node.select("div.tile-info").pop()?;
			let a_person_link_nodes = div_tile_info_node.select("a.person-link");
			let author = a_person_link_nodes
				.iter()
				.map(WNode::text)
				.intersperse(", ".to_string())
				.collect();
			// debug!("author: {author}");

			let div_html_popover_holder_node =
				div_desc_node.select("div.html-popover-holder").pop()?;

			let div_manga_description_node = div_html_popover_holder_node
				.select("div.manga-description")
				.pop()?;
			let description = div_manga_description_node.text();
			// debug!("description: {description}");

			let url = get_manga_url(&id);
			// debug!("url: {}", url);

			let categories = div_html_popover_holder_node
				.select("span.badge-light")
				.iter()
				.map(WNode::text)
				.collect();
			// debug!("categories: {categories:?}");

			// TODO: implement more correct status parsing
			let status = {
				if let [span_node] = &node.select("span.mangaTranslationCompleted")[..] {
					if span_node.text() == "переведено" {
						MangaStatus::Completed
					} else {
						MangaStatus::Unknown
					}
				} else if let [_] = &div_img_node.select("div.manga-updated")[..] {
					MangaStatus::Ongoing
				} else {
					MangaStatus::Unknown
				}
			};
			// debug!("status: {status:?}");

			Some(Manga {
				id,
				cover,
				title,
				author,
				artist: "".to_string(),
				description,
				url,
				categories,
				status,
				nsfw: MangaContentRating::Suggestive,
				viewer: MangaViewer::Rtl,
			})
		})
		.collect();

	Ok(mangas)
}

fn get_manga_page_main_node(html: &WNode) -> Option<WNode> {
	html.select("div.leftContent").pop()
}

pub fn parse_manga(html: &WNode, id: String) -> Result<Manga> {
	let parsing_error = AidokuError {
		reason: AidokuErrorKind::NodeError(NodeError::ParseError),
	};
	let main_node = get_manga_page_main_node(&html).ok_or(parsing_error)?;

	let main_attributes_node = main_node
		.select("div.flex-row")
		.pop()
		.ok_or(parsing_error)?;

	let picture_fororama_node = main_attributes_node.select("div.picture-fotorama").pop();
	let cover = picture_fororama_node
		.and_then(|pfn| pfn.select("img").pop())
		.and_then(|img_node| img_node.attr("src"))
		.unwrap_or_default();

	let names_node = main_node.select("h1.names").pop().ok_or(parsing_error)?;
	let title = names_node
		.select("span")
		.into_iter()
		.map(|name_node| name_node.text())
		.intersperse(" | ".to_string())
		.collect();

	let main_info_node = main_attributes_node
		.select("div.subject-meta")
		.pop()
		.ok_or(parsing_error)?;

	let extract_info_iter = |elem_class, link_type| {
		main_info_node
			.select(format!("span.elem_{elem_class}"))
			.into_iter()
			.flat_map(move |node| {
				node.select(format!("a.{link_type}-link"))
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
		.filter(|mn| {
			if let Some(itemprop) = mn.attr("itemprop") {
				return itemprop == "description";
			}
			false
		})
		.next()
		.and_then(|desc_node| desc_node.attr("content"))
		.unwrap_or_default();

	let url = get_manga_url(&id);

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
		.filter(|sn| {
			if let Some(class_attr) = sn.attr("class") {
				return class_attr
					.split_whitespace()
					.any(|cl| cl.starts_with("text-"));
			}
			false
		})
		.next()
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
		nsfw: MangaContentRating::Suggestive,
		viewer,
	})
}

pub fn get_chapter_url(manga_id: &str, chapter_id: &str) -> String {
	// mtr is 18+ skip
	format!("{BASE_URL}/{manga_id}/{chapter_id}?mtr=true")
}

pub fn parse_chapters(html: &WNode, manga_id: &str) -> Result<Vec<Chapter>> {
	let parsing_error = AidokuError {
		reason: AidokuErrorKind::NodeError(NodeError::ParseError),
	};
	let main_node = get_manga_page_main_node(&html).ok_or(parsing_error)?;

	let chapters = main_node
		.select("div.chapters-link > table > tbody > tr:has(td > a):has(td.date:not(.text-info))")
		.into_iter()
		.filter_map(|chapter_elem| {
			// debug!("chapter_elem: {chapter_elem:?}");
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

			let title = link_elem.text().replace(" новое", "").trim().to_string();

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

			let url = get_chapter_url(&manga_id, &id);

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

pub fn get_page_list(html: &WNode) -> Result<Vec<Page>> {
	let parsing_error = AidokuError {
		reason: AidokuErrorKind::NodeError(NodeError::ParseError),
	};

	let script_text = html
		.select(r"div.reader-controller > script[type=text/javascript]")
		.pop()
		.map(|script_node| script_node.data())
		.ok_or(parsing_error)?;

	let chapters_list_str = script_text
		.find("[[")
		.zip(script_text.find("]]"))
		.map(|(start, end)| &script_text[start..end + 2])
		.ok_or(parsing_error)?;
	debug!("chapters_list_str: {chapters_list_str:?}");

	let urls: Vec<_> = chapters_list_str
		.match_indices("['")
		// extracting parts from ['https://t1.rmr.rocks/', '', "auto/68/88/46/0098.png_res.jpg", 959, 1400] into tuples
		.zip(chapters_list_str.match_indices("\","))
		.filter_map(|((l, _), (r, _))| {
			use itertools::Itertools;
			chapters_list_str[l + 1..r + 1]
				.replace('\'', "")
				.replace('"', "")
				.split(',')
				.map(ToString::to_string)
				.collect_tuple()
		})
		// composing URL
		.map(|(part0, part1, part2)| {
			if part1.is_empty() && part2.starts_with("/static/") {
				format!("{BASE_URL}{part2}")
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

pub fn get_filter_url(filters: &Vec<Filter>, sorting: &Sorting, page: i32) -> Result<String> {
	fn get_handler(operation: &'static str) -> Box<dyn Fn(AidokuError) -> AidokuError> {
		return Box::new(move |err: AidokuError| {
			println!("Error {:?} while {}", err.reason, operation);
			err
		});
	}

	let filter_parts: Vec<_> = filters
		.iter()
		.filter_map(|filter| match filter.kind {
			FilterType::Title => filter
				.value
				.clone()
				.as_string()
				.map_err(get_handler("casting to string"))
				.ok()
				.map(|title| format!("q={}", encode_uri(title.read()))),
			_ => None,
		})
		.collect();

	let offset = format!("offset={}", (page - 1) * SEARCH_OFFSET_STEP);
	let sort = format!("sortType={}", sorting);

	Ok(format!(
		"{}{}",
		BASE_SEARCH_URL,
		chain!(once(offset), once(sort), filter_parts.into_iter())
			.intersperse("&".to_string())
			.collect::<String>()
	))
}

pub fn parse_incoming_url(url: &str) -> Result<DeepLink> {
	let manga_id = match url.find("://") {
		Some(idx) => &url[idx + 3..],
		None => &url[..],
	}
	.split('/')
	.next()
	.ok_or(AidokuError {
		reason: AidokuErrorKind::Unimplemented,
	})?;
	debug!("manga_id: {manga_id}");

	Ok(DeepLink {
		manga: Some(get_manga_details(manga_id.to_string())?),
		chapter: None,
	})
}
