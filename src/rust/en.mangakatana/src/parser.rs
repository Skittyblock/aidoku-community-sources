use aidoku::{
	helpers::substring::Substring,
	std::html::Node,
	std::{String, Vec},
	Chapter, Manga, MangaPageResult, Page,
};

use crate::helper::*;

pub fn parse_manga_list(html: Node, base_url: String) -> MangaPageResult {
	let mut manga: Vec<Manga> = Vec::new();

	// MangaKatana is silly, and the last page of their pagination redirects to the manga
	// page of the last list entry, so if #single_book exists, we know we have been redirected to
	// a manga page and there are no more pages, so just parse the manga details of that last entry
	// and append it to the results and return no more pages
	if !html.select("#single_book").array().is_empty() {
		let url = {
			let raw_url = html.select("link[rel=canonical]").attr("href").read();
			// Converting the raw_url into an absolute url, ie: go-toubun-no-hanayome.18224 => id.18224
			let manga_id = get_manga_id(raw_url);
			get_manga_url(manga_id, base_url)
		};

		manga.push(parse_manga_details(html, url));

		return MangaPageResult {
			manga,
			has_more: false,
		};
	}

	for node in html.select("#book_list > .item").array() {
		let node = node.as_node().expect("Failed to get node");

		let raw_url = node.select(".text .title a").attr("href").read();
		let id = get_manga_id(raw_url);
		let url = get_manga_url(id.clone(), base_url.clone());
		let cover = node.select(".media .wrap_img img").attr("src").read();
		let title = node.select(".text .title a").text().read();
		let description = text_with_newlines(node.select(".text .summary"));

		let status = {
			let status_string = node.select(".media .status").text().read();
			get_manga_status(status_string)
		};

		let mut categories = Vec::new();
		for genre in node.select(".text .genres a").array() {
			let genre = genre.as_node().expect("Failed to get genre node");
			let genre = genre.text().read();
			categories.push(genre);
		}

		let nsfw = get_manga_content_rating(categories.clone());
		let viewer = get_manga_viewer(categories.clone());

		manga.push(Manga {
			id,
			cover,
			title,
			description,
			url,
			categories,
			status,
			nsfw,
			viewer,
			..Default::default()
		});
	}

	// MangaKatana has a bunch of empty pages for some reason, so,
	// if no manga is found, then we know we're on an empty page and
	// there are no more usefull pages even if the pagination button is still there
	let has_more = !html.select(".uk-pagination .next").array().is_empty() && !manga.is_empty();

	MangaPageResult { manga, has_more }
}

pub fn parse_manga_details(html: Node, manga_url: String) -> Manga {
	let id = get_manga_id(manga_url.clone());
	let cover = html.select("#single_book .cover img").attr("src").read();
	let title = html.select("#single_book .info .heading").text().read();
	let author = html
		.select("#single_book .info .meta .author")
		.text()
		.read();
	let description = text_with_newlines(html.select("#single_book .summary p"));

	let status = {
		let status_string = html
			.select("#single_book .info .meta .status")
			.text()
			.read();
		get_manga_status(status_string)
	};

	let mut categories = Vec::new();
	for genre in html.select("#single_book .info .meta .genres a").array() {
		let genre = genre.as_node().expect("Failed to get genre node");
		let genre = genre.text().read();
		categories.push(genre);
	}

	let nsfw = get_manga_content_rating(categories.clone());
	let viewer = get_manga_viewer(categories.clone());

	Manga {
		id,
		cover,
		title,
		author,
		description,
		url: manga_url,
		categories,
		status,
		nsfw,
		viewer,
		..Default::default()
	}
}

pub fn parse_chapter_list(html: Node, base_url: String) -> Vec<Chapter> {
	let mut chapters: Vec<Chapter> = Vec::new();

	for node in html.select("#single_book .chapters tr").array() {
		let node = node.as_node().expect("Failed to get chapter node");

		let raw_url = node.select(".chapter a").attr("href").read();
		let id = get_chapter_id(raw_url.clone());
		let manga_id = get_manga_id(raw_url);
		let url = get_chapter_url(id.clone(), manga_id, base_url.clone());

		let mut title = String::new();
		let chapter = {
			let raw_title = node.select(".chapter a").text().read();
			// If raw title is "Oneshot", then chapter is 0.0
			if raw_title == "Oneshot" {
				title = raw_title;
				0.0
			} else {
				let split_title = raw_title.split_whitespace().collect::<Vec<&str>>();
				// Pull out chatper title from split title
				// and remove any leading characters
				if split_title.len() > 2 {
					if split_title[2] == ":" || split_title[2] == "-" {
						title = split_title[3..].join(" ");
					} else {
						title = split_title[2..].join(" ");
					}
				}

				// Example Title: Chapter 1: A Miracle Appears
				split_title[1]
					.replace(':', "")
					.parse::<f32>()
					.unwrap_or(-1.0)
			}
		};

		let date_updated =
			node.select(".update_time")
				.text()
				.as_date("MMM-dd-yyyy", Some("en-US"), None);

		chapters.push(Chapter {
			id,
			title,
			chapter,
			date_updated,
			url,
			..Default::default()
		});
	}

	chapters
}

pub fn parse_page_list(html: Node) -> Vec<Page> {
	let mut pages: Vec<Page> = Vec::new();

	// Find the script element that contains the image urls
	for script in html.select("script").array() {
		let node = script.as_node().expect("Failed to get script node");
		let script = node.html().read();

		// If script doesn't contain data-src, then it's not the script we want
		if !script.contains("data-src") {
			continue;
		}

		// We are using substring_after_last and substring_before_last because
		// the script contains a dummy array with only one page in it, we don't
		// want that, we want the actual array that contains all the pages.
		// Luckily, the dummy array is always the first array in the script,
		// and the actual array is always the last array in the script.
		let before = script.substring_after_last("=[").unwrap();
		let after = before.substring_before_last(",];").unwrap();

		let urls = after.split(',');

		for url in urls {
			let url = url.replace('\'', "");

			let index = {
				let before = url.substring_after_last('/').unwrap_or("");
				let after = before.substring_before('.').unwrap_or("");
				after.parse::<i32>().unwrap_or(-1)
			};

			pages.push(Page {
				index,
				url,
				..Default::default()
			});
		}
	}

	pages
}
