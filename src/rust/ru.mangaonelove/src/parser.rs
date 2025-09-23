use aidoku::{
	helpers::{substring::Substring, uri::encode_uri},
	prelude::*,
	std::{current_date, String, StringRef, Vec},
	Chapter, Filter, FilterType, Listing, Manga, MangaContentRating, MangaViewer, Page,
};

extern crate alloc;
use alloc::string::ToString;

use crate::{
	constants::PAGE_DIR,
	helpers::{get_base_url, get_manga_id, get_manga_url, parse_status, show_nsfw, show_only_nsfw},
	wrappers::{post, WNode},
};

pub fn parse_lising(html: &WNode, listing: Listing) -> Option<Vec<Manga>> {
	let sidebar_class = match listing.name.as_str() {
		"Популярное" => "c-top-sidebar",
		"Новое" => "c-top-second-sidebar",
		_ => return None,
	};

	let sidebar_node = html.select_one(&format!("div.c-sidebar.{sidebar_class}"))?;

	let allow_manga = |rating: &MangaContentRating| -> bool {
		if show_only_nsfw() {
			if show_nsfw() {
				*rating == MangaContentRating::Nsfw
			} else {
				*rating != MangaContentRating::Nsfw
			}
		} else if show_nsfw() {
			true
		} else {
			*rating != MangaContentRating::Nsfw
		}
	};

	let mangas = sidebar_node
		.select("div.slider__item")
		.iter()
		.filter_map(|manga_node| {
			let thumb_node = manga_node.select_one("div.slider__thumb_item")?;
			let desc_node = manga_node.select_one("div.slider__content_item")?;

			let thumb_link_node = thumb_node.select_one("a")?;
			let url = thumb_link_node.attr("href")?;
			let id = get_manga_id(&url)?;

			let img_node = thumb_link_node.select_one("img")?;
			let cover = match img_node.attr("data-src").or_else(|| img_node.attr("src")) {
				Some(c) => c,
				None => return None,
			};

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
		.filter(|m| allow_manga(&m.nsfw))
		.collect();

	Some(mangas)
}

pub fn parse_search_results(html: &WNode) -> Option<Vec<Manga>> {
	let list_node = html
		.select_one("div.c-page-content div.main-col-inner div.tab-content-wrap div.c-tabs-item")?;

	let allow_manga = |rating: &MangaContentRating| -> bool {
		if show_only_nsfw() {
			if show_nsfw() {
				*rating == MangaContentRating::Nsfw
			} else {
				*rating != MangaContentRating::Nsfw
			}
		} else if show_nsfw() {
			true
		} else {
			*rating != MangaContentRating::Nsfw
		}
	};

	let mangas = list_node
		.select("div.c-tabs-item__content")
		.into_iter()
		.filter_map(|manga_node| {
			let thumb_node = manga_node.select_one("div.tab-thumb")?;
			let summary_node = manga_node.select_one("div.tab-summary")?;

			let title_node = summary_node.select_one("div.post-title")?;
			let content_node = summary_node.select_one("div.post-content")?;

			let extract_from_content = |class_name| {
				content_node
					.select_one(&format!("div.{class_name}"))?
					.select_one("div.summary-content")
					.map(|n| n.text())
			};

			let title_link_node = title_node.select_one("a")?;
			let url = title_link_node.attr("href")?;
			let id = get_manga_id(&url)?;
			let img_node = thumb_node.select_one("img")?;
			let cover = match img_node.attr("data-src").or_else(|| img_node.attr("src")) {
				Some(c) => c,
				None => return None,
			};
			let title = title_link_node.text();
			let author = extract_from_content("mg_author").unwrap_or_default();
			let artist = extract_from_content("mg_artists").unwrap_or_default();
			let categories: Vec<String> = content_node
				.select("div.mg_genres a")
				.iter()
				.map(WNode::text)
				.collect();
			let status = parse_status(&extract_from_content("mg_status")?);
			let nsfw = if categories
				.iter()
				.any(|c| c.contains("18+") || c.contains("Взрослая"))
			{
				MangaContentRating::Nsfw
			} else {
				MangaContentRating::Suggestive
			};

			let manga = Manga {
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
			};

			if allow_manga(&manga.nsfw) {
				Some(manga)
			} else {
				None
			}
		})
		.collect();

	Some(mangas)
}

pub fn parse_manga(html: &WNode, id: String) -> Option<Manga> {
	let main_node = html.select_one("div.profile-manga > div.container > div.row")?;
	let description_node = html.select_one("div.c-page-content div.description-summary")?;
	let summary_node = main_node.select_one("div.tab-summary")?;
	let summary_content_node = summary_node.select_one("div.summary_content")?;
	let content_node = summary_content_node.select_one("div.post-content")?;
	let status_node = summary_content_node.select_one("div.post-status")?;

	let extract_optional_content = |content_type| {
		content_node
			.select_one(&format!("div.{content_type}-content"))
			.map(|type_node| {
				type_node
					.select("a")
					.iter()
					.map(WNode::text)
					.map(|s| s.trim().to_string())
					.collect::<Vec<_>>()
			})
			.unwrap_or_default()
	};

	let get_row_value_by_name = |parent_node: &WNode, row_name| {
		parent_node
			.select("div.post-content_item")
			.into_iter()
			.find(|n| match n.select_one("div.summary-heading") {
				Some(heading) => heading.text().trim() == row_name,
				None => false,
			})
			.and_then(|n| {
				Some(
					n.select_one("div.summary-content")?
						.text()
						.trim()
						.to_string(),
				)
			})
	};

	let cover = summary_node
		.select_one("div.summary_image img")?
		.attr("data-src")
		.or_else(|| {
			summary_node
				.select_one("div.summary_image img")?
				.attr("src")
		})?;
	let url = get_manga_url(&id);
	let title = main_node.select_one("div.post-title > h1")?.text();
	let author = extract_optional_content("authors").join(", ");
	let artist = extract_optional_content("artist").join(", ");

	let categories = extract_optional_content("genres");
	let nsfw = if categories.iter().any(|c| c.contains("18+"))
		|| categories.iter().any(|c| c.contains("Взрослая"))
	{
		MangaContentRating::Nsfw
	} else {
		MangaContentRating::Suggestive
	};
	let status = get_row_value_by_name(&status_node, "Статус")
		.map(|status_str| parse_status(&status_str))
		.unwrap_or_default();
	let viewer = get_row_value_by_name(&content_node, "Тип")
		.map(|manga_type| match manga_type.as_str() {
			"Манхва" => MangaViewer::Scroll,
			"Маньхуа" => MangaViewer::Scroll,
			_ => MangaViewer::default(),
		})
		.unwrap_or_default();
	let description = description_node.text();

	Some(Manga {
		id,
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
	})
}

pub fn parse_chapters(html: &WNode, manga_id: &str) -> Option<Vec<Chapter>> {
	// Prefer inline chapter list if present (as on example.manga.html),
	// otherwise fallback to AJAX-loaded chapters
	let chapter_nodes =
		match html.select_one("div.page-content-listing.single-page ul.main.version-chap") {
			Some(list) => list.select("li.wp-manga-chapter"),
			None => {
				let manga_chapters_holder_node =
					html.select_one("div.c-page-content div#manga-chapters-holder")?;
				let data_id = manga_chapters_holder_node.attr("data-id")?;
				let real_manga_chapters_holder_node = post(
					&format!("{}/wp-admin/admin-ajax.php", get_base_url()),
					&format!("action=manga_get_chapters&manga={data_id}"),
					&[
						("X-Requested-With", "XMLHttpRequest"),
						("Referer", &format!("{}", get_manga_url(manga_id))),
					],
				)
				.ok()?;
				real_manga_chapters_holder_node.select("ul > li.wp-manga-chapter")
			}
		};

	let abs = |l, r| {
		if l > r {
			l - r
		} else {
			r - l
		}
	};

	let chapters = chapter_nodes
		.into_iter()
		.enumerate()
		.filter_map(|(idx, chapter_node)| {
			let url_node = chapter_node.select_one("a")?;
			let url = url_node.attr("href")?;
			let id = url
				.substring_after(&format!("{}/", get_manga_url(manga_id)))?
				.trim_end_matches('/')
				.to_string();
			let title = url_node.text();

			let chapter = {
				let approx_chapter = (idx + 1) as f32;
				let mut possible_chapters: Vec<_> = title
					.split_whitespace()
					.filter_map(|word| word.parse::<f32>().ok())
					.collect();
				match possible_chapters[..] {
					[] => approx_chapter,
					[chap] => chap,
					_ => {
						possible_chapters.sort_by(|&l, &r| {
							abs(l, approx_chapter)
								.partial_cmp(&abs(r, approx_chapter))
								.unwrap()
						});
						possible_chapters.first().cloned().unwrap_or(approx_chapter)
					}
				}
			};

			let extract_multiplier = |metric_str: &&str| {
				if metric_str.starts_with("сек") {
					Some(1)
				} else if metric_str.starts_with("мин") {
					Some(60)
				} else if metric_str.starts_with("час") {
					Some(60 * 60)
				} else if metric_str.starts_with("дн") {
					Some(24 * 60 * 60)
				} else {
					None
				}
			};

			let date_updated = {
				let release_date_node = chapter_node.select_one("span.chapter-release-date")?;
				let normal_release_date = release_date_node.select_one("i").map(|i_node| {
					let txt = i_node.text();
					let parsed1 = StringRef::from(&txt).as_date("dd.MM.yyyy", None, None);
					if parsed1 > 0f64 {
						parsed1
					} else {
						StringRef::from(&txt).as_date("dd-MM-yyyy", None, None)
					}
				});

				let ago_extractor = || {
					release_date_node
						.select_one("a")
						.and_then(|a| a.attr("title"))
						.and_then(|updated_text| {
							if !updated_text.ends_with("ago") {
								return None;
							}
							let spl: Vec<_> = updated_text.split_whitespace().collect();
							let count = spl
								.first()
								.and_then(|count_str| count_str.parse::<f64>().ok())?;

							let metric_mult = spl.get(1).and_then(extract_multiplier)?;

							Some(current_date() - count * (metric_mult as f64))
						})
						.unwrap_or(0f64)
				};

				normal_release_date.unwrap_or_else(ago_extractor)
			};

			// TODO: implement proper volume parsing
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

	Some(chapters)
}

pub fn get_page_list(html: &WNode) -> Option<Vec<Page>> {
	let reader_content_node = html.select_one("div.read-container > div.reading-content")?;
	let page_nodes = reader_content_node.select("div.page-break > img");
	let urls: Vec<_> = page_nodes
		.into_iter()
		.filter_map(|img_node| img_node.attr("src"))
		.map(|url| url.trim().to_string())
		.collect();

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

	Some(format!(
		"{}/{PAGE_DIR}/{page}/?post_type=wp-manga&m_orderby=latest{}",
		get_base_url(),
		filter_addition
	))
}
