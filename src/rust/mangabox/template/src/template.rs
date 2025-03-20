use aidoku::{
	error::Result, prelude::*, std::net::Request, std::ObjectRef, std::String, std::Vec, Chapter,
	DeepLink, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult, MangaViewer,
	Page,
};

use crate::helper::*;

pub fn get_manga_list(
	base_url: &str,
	selector: &str,
	filters: Vec<Filter>,
	page: i32,
	supports_advanced_search: bool,
	search_path: Option<&str>,
	genres: Option<&[&str]>,
) -> Result<MangaPageResult> {
	let url = get_search_url(
		base_url,
		page,
		filters,
		supports_advanced_search,
		search_path,
		genres,
	)?;

	parse_manga_list(&url, base_url, selector, page)
}

pub fn parse_manga_list(
	url: &str,
	base_url: &str,
	selector: &str,
	page: i32,
) -> Result<MangaPageResult> {
	let html = Request::get(url).html()?;

	let mut manga: Vec<Manga> = Vec::new();

	for item in html.select(selector).array() {
		let item_node = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let mut title = item_node.select(".story_name").text().read();
		if title.is_empty() {
			title = item_node.select("a").first().attr("title").read();
		}
		let url = item_node.select("a").first().attr("href").read();
		let id = url.strip_prefix(base_url).unwrap_or(&url).into();
		let cover = item_node.select("img").first().attr("src").read();
		manga.push(Manga {
			id,
			cover,
			title,
			..Default::default()
		});
	}

	let last_page_string = html.select("a.page-last").text().read();
	let mut last_page = 1;
	if !last_page_string.is_empty() {
		last_page = String::from(&last_page_string[5..last_page_string.len() - 1])
			.parse::<i32>()
			.unwrap_or(1);
	}

	Ok(MangaPageResult {
		manga,
		has_more: page < last_page,
	})
}

pub fn get_manga_listing(
	base_url: &str,
	selector: &str,
	listing: Listing,
	page: i32,
	supports_advanced_search: bool,
	search_path: Option<&str>,
	genres: Option<&[&str]>,
) -> Result<MangaPageResult> {
	let mut filters: Vec<Filter> = Vec::new();
	let mut selection = ObjectRef::new();

	selection.set(
		"index",
		match listing.name.as_str() {
			"Latest Updates" => 1i32.into(),
			"Top Manga" => 2i32.into(),
			_ => 0i32.into(),
		},
	);
	filters.push(Filter {
		kind: FilterType::Sort,
		name: String::from("Sort"),
		value: selection.0.clone(),
		object: ObjectRef(selection.0),
	});

	get_manga_list(
		base_url,
		selector,
		filters,
		page,
		supports_advanced_search,
		search_path,
		genres,
	)
}

pub fn get_manga_details(
	id: String,
	base_url: &str,
	description_selector: Option<&str>,
) -> Result<Manga> {
	let url = if id.starts_with("http") {
		id.clone()
	} else {
		format!("{base_url}{id}")
	};
	let html = Request::get(&url).html()?;

	let details = html.select("div.manga-info-top, div.panel-story-info");
	let title = details.select("h1").text().read();
	let cover = details
		.select("div.manga-info-pic img, span.info-image img")
		.attr("src")
		.read();
	let author = join_string_array(
		details
			.select("li:contains(author) a, td:containsOwn(author) + td a")
			.array(),
		", ",
	);
	let description = html
		.select(description_selector.unwrap_or("div#noidungm, div#panel-story-info-description"))
		.text()
		.read()
		.replace(&format!("{title} summary:"), "")
		.trim()
		.into();

	let mut categories = Vec::new();
	let mut nsfw = MangaContentRating::Safe;
	let mut viewer = MangaViewer::Rtl;
	for node in details
		.select("div.manga-info-top li:contains(genres) a, td:containsOwn(genres) + td a")
		.array()
	{
		let category = node.as_node().expect("node array").text().read();
		match category.as_str() {
			"Adult" | "Ecchi" | "Mature" | "Smut" => nsfw = MangaContentRating::Nsfw,
			"Webtoons" | "Manhua" | "Manhwa" => viewer = MangaViewer::Scroll,
			_ => (),
		}
		categories.push(category);
	}

	let status = status_from_string(
		details
			.select("li:contains(status), td:containsOwn(status) + td")
			.text()
			.read(),
	);

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

pub fn get_chapter_list(
	manga_id: String,
	base_url: &str,
	date_format: &str,
) -> Result<Vec<Chapter>> {
	let url = if manga_id.starts_with("http") {
		manga_id
	} else {
		format!("{base_url}{manga_id}")
	};
	let html = Request::get(&url).html()?;

	let mut chapters: Vec<Chapter> = Vec::new();

	for chapter in html
		.select("div.chapter-list div.row, ul.row-content-chapter li")
		.array()
	{
		let chapter_node = chapter.as_node().expect("node array");
		let link = chapter_node.select("a");

		let title = strip_default_chapter_title(link.text().read());
		let url = link.attr("href").read();
		let id = url.strip_prefix(base_url).unwrap_or(&url).into();
		let chapter = get_chapter_number(&url);
		let date_updated = chapter_node
			.select("span")
			.attr("title")
			.0
			.as_date(date_format, None, None)
			.unwrap_or(0.0);

		chapters.push(Chapter {
			id,
			title,
			chapter,
			date_updated,
			url,
			lang: String::from("en"),
			..Default::default()
		});
	}

	Ok(chapters)
}

pub fn get_page_list(chapter_id: String, base_url: &str) -> Result<Vec<Page>> {
	let url = if chapter_id.starts_with("http") {
		chapter_id
	} else {
		format!("{base_url}{chapter_id}")
	};

	let html = Request::get(url).html()?;

	Ok(html
		.select("div.container-chapter-reader > img")
		.array()
		.enumerate()
		.map(|(i, page)| {
			let url = page.as_node().expect("node array").attr("src").read();
			Page {
				index: i as i32,
				url,
				..Default::default()
			}
		})
		.collect())
}

pub fn modify_image_request(base_url: &str, request: Request) {
	request.header("Referer", base_url);
}

pub fn handle_url(url: String, base_url: &str) -> Result<DeepLink> {
	let id = url.strip_prefix(base_url).unwrap_or(&url);
	Ok(DeepLink {
		manga: get_manga_details(id.into(), base_url, None).ok(),
		chapter: None,
	})
}
