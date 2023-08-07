#![no_std]

mod helper;

extern crate alloc;

use aidoku::{
	error::Result,
	prelude::*,
	std::net::Request,
	std::{net::HttpMethod, String, Vec},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};
use helper::USER_AGENT;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut manga_arr: Vec<Manga> = Vec::new();
	let mut total: i32 = 1;

	let mut query: Option<String> = None;
	let mut sort: String = String::new();
	let tag_list = helper::tag_list();
	let mut tags: Vec<String> = Vec::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => query = Some(helper::urlencode(filter.value.as_string()?.read())),
			FilterType::Select => {
				if filter.name.as_str() == "Tags" {
					let index = filter.value.as_int()? as usize;
					match index {
						0 => continue,
						_ => tags.push(String::from(tag_list[index])),
					}
				}
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int().unwrap_or(0);

				let option = match index {
					0 => "latest",
					1 => "popular",
					_ => "",
				};
				sort = String::from(option)
			}
			_ => continue,
		}
	}

	let url = helper::build_search_url(query, tags.clone(), sort, page);

	let html = Request::new(url.as_str(), HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.html()?;

	for result in html.select(".lc_galleries .thumb").array() {
		let res_node = result.as_node().expect("Failed to get node");
		let a_tag = res_node.select(".caption .g_title a");
		let title = a_tag.text().read();
		let href = a_tag.attr("href").read();
		let id = helper::get_gallery_id(href);
		let cover = res_node.select(".inner_thumb img").attr("src").read();
		let id_str = helper::i32_to_string(id);

		manga_arr.push(Manga {
			id: id_str,
			cover,
			title,
			status: MangaStatus::Completed,
			nsfw: MangaContentRating::Nsfw,
			viewer: MangaViewer::Rtl,
			..Default::default()
		})
	}

	for paging_res in html.select(".pagination .page-item a").array() {
		let paging = paging_res.as_node().expect("Failed to get node");
		let href = paging.attr("href").read();
		if href == "#" {
			continue;
		}
		let href_parts = href.split('/').collect::<Vec<&str>>();

		// get second last part in href
		let last_str = String::from(href_parts[href_parts.len() - 1]);

		if last_str.starts_with("?q=") {
			if !last_str.contains("&page=") {
				continue;
			}
			let last_str_parts = last_str.split('&').collect::<Vec<&str>>();

			let page_str = String::from(last_str_parts[1]);

			let page_str_parts = page_str.split('=').collect::<Vec<&str>>();
			let page_num_str = String::from(page_str_parts[1]);
			let page_num = helper::numbers_only_from_string(page_num_str);

			if page_num > total {
				total = page_num;
			}

			continue;
		}

		let num_str = String::from(href_parts[href_parts.len() - 2]);

		let num = helper::numbers_only_from_string(num_str);

		if num > total {
			total = num;
		}
	}

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: page < total,
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("https://hentaifox.com/gallery/{}", id);
	let html = Request::new(url.as_str(), HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.html()?;

	let cover = html
		.select(".gallery_top .gallery_left img")
		.attr("src")
		.read();
	let title = html.select(".gallery_top .gallery_right h1").text().read();
	let author_str = html
		.select(".gallery_top .gallery_right .artists li a")
		.first()
		.text()
		.read();
	let author = helper::only_chars_from_string(author_str);
	let artist = String::new();
	let description = String::new();
	let mut categories: Vec<String> = Vec::new();
	for tags_arr in html
		.select(".gallery_top .gallery_right .tags li a")
		.array()
	{
		let tags = tags_arr.as_node().expect("Failed to get node");
		let tag = tags.attr("href").read();
		let tag_str = helper::get_tag_slug(tag);

		categories.push(tag_str);
	}

	let manga = Manga {
		id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories,
		status: MangaStatus::Completed,
		nsfw: MangaContentRating::Nsfw,
		viewer: MangaViewer::Rtl,
	};
	Ok(manga)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("https://hentaifox.com/gallery/{}", id);

	Ok(Vec::from([Chapter {
		id,
		title: String::from("Chapter 1"),
		volume: -1.0,
		chapter: 1.0,
		url,
		date_updated: 0.0,
		scanlator: String::new(),
		lang: String::from("en"),
	}]))
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("https://hentaifox.com/gallery/{chapter_id}");
	let html = Request::new(url.as_str(), HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.html()?;

	let g_id = html.select("#load_id").attr("value").read();
	let img_dir = html.select("#load_dir").attr("value").read();
	let total_pages = html.select("#load_pages").attr("value").read();

	let mut pages: Vec<Page> = Vec::new();

	let total = helper::numbers_only_from_string(total_pages);
	for i in 1..=total {
		let img_url = format!("https://i2.hentaifox.com/{img_dir}/{g_id}/{i}.jpg");
		pages.push(Page {
			index: i,
			url: img_url,
			base64: String::new(),
			text: String::new(),
		})
	}

	Ok(pages)
}
