use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	helpers::{substring::Substring, uri::encode_uri},
	prelude::*,
	std::{String, StringRef, Vec},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaStatus, MangaViewer,
	Page,
};

extern crate alloc;
use alloc::string::ToString;

use itertools::chain;

use crate::{
	constants::SEARCH_OFFSET_STEP, get_manga_details, helpers, sorting::Sorting, wrappers::WNode,
};

pub fn parse_search_results(html: &WNode) -> Result<Vec<Manga>> {
	let nodes = html.select("div.tile");

	let mangas: Vec<_> = nodes
		.into_iter()
		.filter_map(|node| {
			let div_img_node = node.select("div.img").pop()?;

			let id = {
				let a_non_hover_node = div_img_node.select("a.non-hover").pop()?;
				a_non_hover_node
					.attr("href")?
					.trim_start_matches('/')
					.to_string()
			};

			let img_node = div_img_node.select("img").pop()?;
			let cover = img_node.attr("original")?;
			let title = img_node.attr("title")?;

			let div_desc_node = node.select("div.desc").pop()?;

			let div_tile_info_node = div_desc_node.select("div.tile-info").pop()?;
			let a_person_link_nodes = div_tile_info_node.select("a.person-link");
			let author = a_person_link_nodes
				.iter()
				.map(WNode::text)
				.intersperse(", ".to_string())
				.collect();

			let div_html_popover_holder_node =
				div_desc_node.select("div.html-popover-holder").pop()?;

			let div_manga_description_node = div_html_popover_holder_node
				.select("div.manga-description")
				.pop()?;
			let description = div_manga_description_node.text();

			let url = helpers::get_manga_url(&id);

			let mut categories: Vec<String> = Vec::new();
			categories.extend(div_tile_info_node.select("a.badge").iter().map(WNode::text));
			categories.extend(
				div_html_popover_holder_node
					.select("span.elem_genre")
					.iter()
					.map(WNode::text),
			);
			categories.extend(
				div_html_popover_holder_node
					.select("span.elem_tag")
					.iter()
					.map(WNode::text),
			);

			let status = {
				let has_completed_badge = !node.select("span.mangaTranslationCompleted").is_empty()
					|| !node.select("span.mangaCompleted").is_empty();
				if has_completed_badge {
					MangaStatus::Completed
				} else if let [_] = &div_img_node.select("div.manga-updated")[..] {
					MangaStatus::Ongoing
				} else {
					MangaStatus::Unknown
				}
			};

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
				nsfw: MangaContentRating::default(),
				viewer: MangaViewer::Rtl,
			})
		})
		.collect();

	Ok(mangas)
}

fn get_manga_page_main_node(html: &WNode) -> Result<WNode> {
	html.select("div.leftContent")
		.pop()
		.ok_or(helpers::create_parsing_error())
}

pub fn parse_manga(html: &WNode, id: String) -> Result<Manga> {
	let parsing_error = helpers::create_parsing_error();

	let main_node = get_manga_page_main_node(html)?;

	let main_attributes_node = main_node
		.select("div.flex-row")
		.pop()
		.ok_or(parsing_error)?;

	let picture_fororama_node = main_attributes_node.select("div.picture-fotorama").pop();
	let cover = picture_fororama_node
		.and_then(|pfn| {
			let imgs = pfn.select("img");
			imgs.into_iter().next()
		})
		.and_then(|img_node| {
			img_node
				.attr("data-full")
				.or_else(|| img_node.attr("data-thumb"))
				.or_else(|| img_node.attr("src"))
		})
		.map(|url| {
			if url.contains("://") {
				url
			} else {
				format!("https:{url}")
			}
		})
		.unwrap_or_default();

	let names_node = main_node.select("h1.names").pop().ok_or(parsing_error)?;
	let title = names_node
		.select("span.name")
		.pop()
		.ok_or(parsing_error)?
		.text();

	let main_info_node = main_attributes_node
		.select("div.subject-meta")
		.pop()
		.ok_or(parsing_error)?;

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

	let url = helpers::get_manga_url(&id);

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

	let mut categories: Vec<String> = Vec::new();
	if let Some(category) = category_opt {
		categories.push(category);
	}
	// Genres
	categories.extend(
		main_info_node
			.select("a.elem_genre")
			.iter()
			.map(WNode::text),
	);
	categories.extend(
		main_info_node
			.select("span.elem_genre")
			.iter()
			.map(WNode::text),
	);
	// Tags
	categories.extend(main_info_node.select("a.elem_tag").iter().map(WNode::text));
	categories.extend(
		main_info_node
			.select("span.elem_tag")
			.iter()
			.map(WNode::text),
	);

	let badge_texts: Vec<String> = main_info_node
		.select("p span.badge")
		.iter()
		.map(WNode::text)
		.map(|s| s.to_lowercase())
		.collect();
	let status = if badge_texts.iter().any(|t| {
		t.contains("выпуск завершён") || t.contains("завершён") || t.contains("переведено")
	}) {
		MangaStatus::Completed
	} else if badge_texts
		.iter()
		.any(|t| t.contains("выпуск продолжается") || t.contains("переводится"))
	{
		MangaStatus::Ongoing
	} else if badge_texts.iter().any(|t| {
		t.contains("перевод приостановлен")
			|| t.contains("выпуск остановлен")
			|| t.contains("заморожен")
	}) {
		MangaStatus::Hiatus
	} else {
		MangaStatus::Unknown
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

pub fn parse_chapters(html: &WNode, manga_id: &str) -> Result<Vec<Chapter>> {
	let main_node = get_manga_page_main_node(html)?;

	let chapters = main_node
		.select(
			"div[class~=chapters] > table > tbody > tr:has(td > a):has(td.date:not(.text-info))",
		)
		.into_iter()
		.filter_map(|chapter_elem| {
			let link_elem = chapter_elem.select("a.chapter-link").pop()?;

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

			let url = helpers::get_chapter_url(manga_id, &id);

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
	let parsing_error = helpers::create_parsing_error();

	let script_text = html
		.select(r"div.reader-controller > script[type=text/javascript]")
		.pop()
		.map(|script_node| script_node.data())
		.ok_or(parsing_error)
		.map(|mut text| {
			text.replace_range(0..text.find("rm_h.readerDoInit(").unwrap_or_default(), "");
			text
		})?;

	let chapters_list_str = script_text
		.find("[[")
		.zip(script_text.find("]]"))
		.map(|(start, end)| &script_text[start..end + 2])
		.ok_or(parsing_error)?;

	let urls: Vec<_> = chapters_list_str
		.match_indices("['")
		.zip(chapters_list_str.match_indices("\","))
		.filter_map(|((l, _), (r, _))| {
			use itertools::Itertools;
			chapters_list_str[l + 1..r + 1]
				.replace(['\'', '"'], "")
				.split(',')
				.map(ToString::to_string)
				.collect_tuple()
		})
		.map(|(part0, part1, part2)| {
			if part1.is_empty() && part2.starts_with("/static/") {
				format!("{}{}", helpers::get_base_url(), part2)
			} else if part1.starts_with("/manga/") {
				format!("{part0}{part2}")
			} else {
				format!("{part0}{part1}{part2}")
			}
		})
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

pub fn get_filter_url(filters: &[Filter], sorting: &Sorting, page: i32) -> Result<String> {
	let mut params: Vec<String> = Vec::new();

	params.push(format!("offset={}", (page - 1) * SEARCH_OFFSET_STEP));
	params.push(format!("sortType={}", sorting));

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(title_ref) = filter.value.clone().as_string() {
					params.push(format!("q={}", encode_uri(title_ref.read())));
				}
			}
			FilterType::Genre => {
				if let Ok(id_ref) = filter.object.get("id").as_string() {
					let id = id_ref.read();
					match filter.value.as_int().unwrap_or(-1) {
						0 => params.push(format!("{}=out", id)), // excluded
						1 => params.push(format!("{}=in", id)),  // included
						_ => {}
					}
				}
			}
			FilterType::Check => {
				if let Ok(id_ref) = filter.object.get("id").as_string() {
					let id = id_ref.read();
					// Any checked option => add `=in`
					if filter.value.as_int().unwrap_or(0) != 0 {
						params.push(format!("{}=in", id));
					}
				}
			}
			_ => {}
		}
	}

	params.sort_by(|a, b| {
		let a_is_q = a.starts_with("q=");
		let b_is_q = b.starts_with("q=");
		b_is_q.cmp(&a_is_q)
	});

	Ok(format!(
		"{}{}",
		helpers::get_base_search_url(),
		params.join("&")
	))
}

pub fn parse_incoming_url(url: &str) -> Result<DeepLink> {
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
