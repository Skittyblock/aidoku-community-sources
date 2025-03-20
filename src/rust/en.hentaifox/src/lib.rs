#![no_std]

mod helper;

extern crate alloc;
use alloc::string::ToString;

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

	let has_more = !manga_arr.is_empty();

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more,
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
		chapter: 1.0,
		url,
		lang: String::from("en"),
		..Default::default()
	}]))
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("https://hentaifox.com/gallery/{chapter_id}");
	let html = Request::new(url.as_str(), HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.html()?;

	let content = html.select("body > script").first().data().read();

	let start_pattern = r#"parseJSON('"#;
	let end_pattern = r#"');"#;

	let json = if let Some(start_index) = content.find(start_pattern) {
		let start_index = start_index + start_pattern.len();
		if let Some(end_index) = content[start_index..].find(end_pattern) {
			let end_index = start_index + end_index;
			Some(&content[start_index..end_index])
		} else {
			None
		}
	} else {
		None
	};

	let json = json
		.and_then(|json| aidoku::std::json::parse(json).ok())
		.and_then(|json| json.as_object().ok());

	let g_id = html.select("#load_id").attr("value").read();
	let img_dir = html.select("#load_dir").attr("value").read();
	let total_pages = html.select("#load_pages").attr("value").read();

	let mut pages: Vec<Page> = Vec::new();

	let total = helper::numbers_only_from_string(total_pages);
	for i in 1..=total {
		let ext = if let Some(page_str) = json
			.as_ref()
			.and_then(|json| json.get(&i.to_string()).as_string().ok())
			.map(|s| s.read())
		{
			match page_str.chars().next().unwrap_or('j') {
				'p' => "png",
				'b' => "bmp",
				'g' => "gif",
				'w' => "webp",
				_ => "jpg",
			}
		} else {
			"jpg"
		};
		let url = format!("https://i2.hentaifox.com/{img_dir}/{g_id}/{i}.{ext}");
		pages.push(Page {
			index: i,
			url,
			..Default::default()
		})
	}

	Ok(pages)
}
