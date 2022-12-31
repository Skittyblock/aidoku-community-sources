#![no_std]

mod helper;
extern crate alloc;
use aidoku::{
	error::Result, prelude::*, std::net::HttpMethod, std::net::Request, std::*, Chapter, DeepLink,
	Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page,
};
use alloc::vec;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut manga_arr = Vec::new();
	let mut total: i32 = 1;

	let mut query = String::new();
	let mut sort = String::new();
	let mut ascending = false;

	let mut included_tags: Vec<String> = Vec::new();
	let mut excluded_tags: Vec<String> = Vec::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = helper::urlencode(filter.value.as_string()?.read());
			}
			FilterType::Genre => {
				if let Ok(tag_id) = filter.object.get("id").as_string() {
					match filter.value.as_int().unwrap_or(-1) {
						0 => excluded_tags.push(tag_id.read()),
						1 => included_tags.push(tag_id.read()),
						_ => continue,
					}
				}
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				ascending = value.get("ascending").as_bool().unwrap_or(false);
				let index = value.get("index").as_int().unwrap_or(0);
				let option = match index {
					0 => "id",
					1 => "title",
					2 => "created_at",
					3 => "published_at",
					4 => "pages",
					_ => "",
				};
				sort = String::from(option)
			}
			_ => continue,
		}
	}

	let url = helper::build_search_url(query, sort, included_tags, excluded_tags, ascending, page);

	let html = Request::new(url.as_str(), HttpMethod::Get).html();

	for result in html.select(".entries .entry a").array() {
		let result_node = result.as_node();
		let manga_url = result_node.attr("href").read();
		if manga_url.is_empty() {
			continue;
		}

		let title = result_node.select(".metadata h3.title span").text().read();

		let manga_id = helper::get_manga_id_from_path(&manga_url);
		let cover = helper::get_cover_url(&manga_id);

		manga_arr.push(Manga {
			id: manga_id,
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: manga_url,
			categories: Vec::new(),
			status: MangaStatus::Completed,
			nsfw: MangaContentRating::Nsfw,
			viewer: MangaViewer::Rtl,
		});
	}

	// check if paging node exists
	let paging = html.select("nav.pagination ul li.last a");
	if !paging.html().read().is_empty() {
		let last_page = helper::get_page(paging.last().attr("href").read());

		if last_page > total {
			total = last_page
		}
	}

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: page < total,
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("https://koushoku.org/archive/{}", id);
	let html = Request::new(url.as_str(), HttpMethod::Get).html();

	let title = html
		.select("main#archive .wrapper section.metadata h1.title")
		.first()
		.text()
		.read();

	let cover = html
		.select("main#archive .wrapper aside figure.thumbnail a img")
		.attr("src")
		.read();

	let author = html
		.select("main#archive .wrapper section.metadata .artists td a")
		.text()
		.read();

	let categories = html
		.select("main#archive .wrapper section.metadata .tags a")
		.array()
		.map(|tag| tag.as_node().text().read())
		.collect::<Vec<String>>();

	let share_url = format!("https://koushoku.org/archive/{}", &id);

	Ok(Manga {
		id,
		title,
		author,
		cover,
		artist: String::new(),
		description: String::new(),
		url: share_url,
		categories,
		status: MangaStatus::Completed,
		nsfw: MangaContentRating::Nsfw,
		viewer: MangaViewer::Rtl,
	})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("https://koushoku.org/archive/{}", id);
	let html = Request::new(url.as_str(), HttpMethod::Get).html();

	let created_at = html
		.select("main#archive .wrapper section.metadata .createdAt td[data-unix]")
		.first()
		.attr("data-unix")
		.read();

	let date_updated = created_at.parse::<f64>().unwrap_or(-1.0);

	Ok(vec![Chapter {
		id,
		title: String::new(),
		volume: -1.0,
		chapter: 1.0,
		url,
		date_updated,
		scanlator: String::new(),
		lang: String::from("en"),
	}])
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	let url = format!("https://koushoku.org/archive/{}", id);

	let mut pages: Vec<Page> = Vec::new();

	let html = Request::new(url.as_str(), HttpMethod::Get).html();

	let pages_total_str = html
		.select("main#archive .wrapper section.metadata .pages td")
		.array()
		.nth(1)
		.unwrap()
		.as_node()
		.text()
		.read();
	let pages_total = pages_total_str.parse::<i32>().unwrap_or(0);

	for i in 1..=pages_total {
		let img_url = format!("https://ksk-h7glm2.xyz/data/{}/{}/512.png", id, i);

		pages.push(Page {
			index: i as i32,
			url: img_url,
			base64: String::new(),
			text: String::new(),
		});
	}

	Ok(pages)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let id = helper::get_manga_id(url);
	let manga = get_manga_details(id)?;
	Ok(DeepLink {
		manga: Some(manga),
		chapter: None,
	})
}
