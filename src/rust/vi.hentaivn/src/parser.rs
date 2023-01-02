use alloc::string::ToString;

use aidoku::{
	error::Result,
	helpers::{cfemail::decode_cfemail, node::NodeHelpers},
	prelude::format,
	std::{html::Node, String, Vec},
	Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page,
};

pub fn parse_search_page(document: Node, includes_non_hen: bool) -> Result<MangaPageResult> {
	decode_cfemail(&document);

	let nodes = document.select("li.search-li");
	let elems = nodes.array();
	let mut manga: Vec<Manga> = Vec::with_capacity(elems.len());

	for elem in elems {
		if let Ok(node) = elem.as_node() {
			let url_elem = node.select("div.search-des a");
			let id = url_elem.attr("href").read().replace('/', "");

			let title_elem = node.select("div.search-des a b");

			let img_elem = node.select("div.search-img img");

			manga.push(Manga {
				id,
				cover: img_elem.attr("abs:src").read(),
				title: title_elem.text().read(),
				nsfw: if includes_non_hen {
					MangaContentRating::Suggestive
				} else {
					MangaContentRating::Nsfw
				},
				..Default::default()
			})
		}
	}

	Ok(MangaPageResult {
		manga,
		has_more: !document
			.select("ul.pagination > li:contains(Cuối)")
			.array()
			.is_empty(),
	})
}

pub fn parse_new_or_complete_page(document: Node) -> Result<MangaPageResult> {
	decode_cfemail(&document);

	let nodes = document.select("li.item");
	let elems = nodes.array();
	let mut manga: Vec<Manga> = Vec::with_capacity(elems.len());

	for elem in elems {
		if let Ok(node) = elem.as_node() {
			let url_elem = node.select("div.box-description a:first-child");
			let id = url_elem.attr("href").read().replace('/', "");

			let tag_elems = node.select("div.box-description b.info:contains(Thể Loại) ~ span");
			let tags = tag_elems
				.array()
				.filter_map(|tag_elem| {
					if let Ok(tag_node) = tag_elem.as_node() {
						Some(tag_node.text().read())
					} else {
						None
					}
				})
				.collect::<Vec<_>>();

			let img_elem = node.select("div.box-cover img");

			manga.push(Manga {
				id,
				cover: img_elem.attr("data-src").read(),
				title: url_elem.text().read(),
				nsfw: if tags.iter().any(|x| x == "Non-hen") {
					MangaContentRating::Suggestive
				} else {
					MangaContentRating::Nsfw
				},
				..Default::default()
			})
		}
	}
	Ok(MangaPageResult {
		manga,
		has_more: !document
			.select("ul.pagination > li:contains(Next)")
			.array()
			.is_empty(),
	})
}

pub fn parse_manga_details(id: String, document: Node) -> Result<Manga> {
	decode_cfemail(&document);

	let title_elem = document.select("div.page-info h1[itemprop=name] a");
	let title = title_elem.text().read().split(" - ").collect::<Vec<_>>()[0].to_string();
	let url = title_elem.attr("abs:href").read();

	let author_elem = document.select("span.info:contains(Tác giả) + span");
	let author = author_elem.text().read().trim().to_string();

	let cover_elem = document.select("div.page-ava img");
	let cover = cover_elem.attr("abs:src").read();

	let status_elem = document.select("span.info:contains(Tình Trạng) + span");
	let status = if status_elem.text().read() == "Đã hoàn thành" {
		MangaStatus::Completed
	} else {
		MangaStatus::Ongoing
	};

	let mut nsfw = MangaContentRating::Nsfw;
	let mut viewer = MangaViewer::Rtl;
	let category_elems = document.select("a.tag");
	let categories = category_elems
		.array()
		.filter_map(|elem| {
			if let Ok(node) = elem.as_node() {
				let category = node.text().read();
				if category == "Non-hen" {
					nsfw = MangaContentRating::Suggestive
				}
				if category == "Manhua" || category == "Manhwa" || category == "Webtoon" {
					viewer = MangaViewer::Scroll
				}
				Some(category)
			} else {
				None
			}
		})
		.collect::<Vec<_>>();

	let mut description = String::new();
	let kvp_lines = document.select("div.page-info p:has(span.info)");
	for kvp_line in kvp_lines.array() {
		if let Ok(kvp_node) = kvp_line.as_node() {
			let keys = kvp_node.select("span.info");
			let values = kvp_node.select("span:not(.info)");

			for (key_ref, value_ref) in keys.array().zip(values.array()) {
				if let Ok(key_node) = key_ref.as_node()
				   && let Ok(value_node) = value_ref.as_node() {
					let key = key_node.text().read();

					if ["Tác giả", "Tình Trạng", "Thể Loại", "Nội dung"].iter().any(|k| key.contains(k)) {
						continue;
					}

					let value = value_node.text().read();
					description.push_str(&format!("{key} {value}\n"));

				}
			}
		}
	}

	let description_elem = document.select("p:contains(Nội dung) + p");
	description.push_str(&description_elem.text_with_newlines());

	Ok(Manga {
		id,
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

pub fn parse_chapter_list(document: Node) -> Result<Vec<Chapter>> {
	decode_cfemail(&document);

	let row_elems = document.select("table.listing tbody tr");
	let rows = row_elems.array();
	let mut chapters = Vec::with_capacity(rows.len());

	for (idx, row) in rows.rev().enumerate() {
		if let Ok(node) = row.as_node() {
			let url_elem = node.select("td:first-child a");
			let id = url_elem.attr("href").read().replace('/', "");

			let title_elem = node.select("h2.chuong_t");
			let title_raw = title_elem.text().read();

			let mut chapter: f32 = if title_raw.to_lowercase().contains("Oneshot")
				|| title_raw.to_lowercase().contains("1shot")
			{
				-1.0
			} else {
				idx as f32
			};
			let title_parts = title_raw
				.split(|v| v == '-' || v == ':')
				.collect::<Vec<_>>();

			let title = if title_parts.len() > 1 {
				title_parts[1].trim().to_string()
			} else {
				title_raw.clone()
			};

			if title_parts[0].contains("Chap") {
				let chapter_raw = title_parts[0]
					.split(char::is_whitespace)
					.last()
					.expect("chapter number");
				chapter = chapter_raw.parse::<f32>().unwrap_or(idx as f32);
			}

			let date_updated_elem = node.select("td:nth-child(2)");
			let date_updated = date_updated_elem.text().as_date(
				"dd/MM/yyyy",
				Some("en_US"),
				Some("Asia/Ho_Chi_Minh"),
			);
			chapters.push(Chapter {
				id,
				chapter,
				title,
				date_updated,
				url: url_elem.attr("abs:href").read(),
				..Default::default()
			})
		}
	}
	chapters.reverse();
	Ok(chapters)
}

pub fn parse_page_list(document: Node, selector: Option<&str>) -> Result<Vec<Page>> {
	let page_elems = if let Some(sel) = selector {
		document.select(sel)
	} else {
		document.select("div#image img")
	};

	Ok(page_elems
		.array()
		.enumerate()
		.filter_map(|(index, elem)| {
			if let Ok(node) = elem.as_node() {
				Some(Page {
					index: index as i32,
					url: node.attr("abs:src").read(),
					..Default::default()
				})
			} else {
				None
			}
		})
		.collect::<Vec<_>>())
}
