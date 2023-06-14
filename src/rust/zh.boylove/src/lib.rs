#![no_std]
use aidoku::{
	error::Result,
	helpers::{
		substring::Substring,
		uri::{encode_uri, QueryParameters},
	},
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	Page,
};

extern crate alloc;
use alloc::string::ToString;

const DOMAIN: &str = "https://boylove.cc";

const API_PATH: &str = "/home/api/";

const HTML_PATH: &str = "/home/book/";
const MANGA_PATH: &str = "index/id/";
const CHAPTER_PATH: &str = "capter/id/";

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36";

fn get(url: String) -> Request {
	Request::get(url)
		.header("Referer", DOMAIN)
		.header("User-Agent", USER_AGENT)
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let manga_list_url = get_filtered_url(filters, page)?;
	let manga_list_json = get(manga_list_url).json()?;
	let manga_list_object = manga_list_json.as_object()?;
	let result = manga_list_object.get("result").as_object()?;

	let mut manga: Vec<Manga> = Vec::new();

	for value in result.get("list").as_array()? {
		let manga_object = value.as_object()?;

		// if manga_object.get("lanmu_id").as_int().unwrap_or(0) == 5 {
		// 	continue;
		// }
		// There's an ad whose lanmu_id is not 5
		let keyword = manga_object.get("keyword").as_string()?.read();
		if keyword.contains("公告") {
			continue;
		}

		let id = manga_object.get("id").as_int()?.to_string();

		let cover_path = manga_object.get("image").as_string()?.read();
		let cover = format!("{}{}", DOMAIN, cover_path);

		let title = manga_object.get("title").as_string()?.read();

		let artist = manga_object
			.get("auther")
			.as_string()?
			.read()
			.replace('&', "、");

		let description = manga_object.get("desc").as_string()?.read();

		let url = format!("{}{}{}{}", DOMAIN, HTML_PATH, MANGA_PATH, id);

		let categories: Vec<String> = keyword
			.split(',')
			.filter(|tag| !tag.is_empty())
			.map(|tag| tag.to_string())
			.collect();

		let status = match manga_object.get("mhstatus").as_int()? {
			0 => MangaStatus::Ongoing,
			1 => MangaStatus::Completed,
			_ => MangaStatus::Unknown,
		};

		let nsfw = match categories.contains(&"清水".to_string()) {
			true => MangaContentRating::Safe,
			false => MangaContentRating::Nsfw,
		};

		manga.push(Manga {
			id,
			cover,
			title,
			author: artist.clone(),
			artist,
			description,
			url,
			categories,
			status,
			nsfw,
			..Default::default()
		})
	}

	let has_more = !result.get("lastPage").as_bool()?;

	Ok(MangaPageResult { manga, has_more })
}

fn get_filtered_url(filters: Vec<Filter>, page: i32) -> Result<String> {
	const FILTER_STATUS: [u8; 3] = [2, 0, 1];

	let mut url = format!("{}{}", DOMAIN, API_PATH);

	let mut filter_status_index = 0;
	let mut filter_content_rating = 0;
	let mut filter_tags_vec: Vec<String> = Vec::new();
	let mut sort_by = 1;

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				let search_str = filter.value.as_string()?;

				let mut query = QueryParameters::new();
				query.push("keyword", Some(search_str.read().as_str()));
				// type=[1: Manga, 2: Novel]
				query.push("type", Some("1"));
				query.push("pageNo", Some(page.to_string().as_str()));

				let searching_path = format!("searchk?{}", query);
				url.push_str(searching_path.as_str());

				return Ok(url);
			}

			FilterType::Select => {
				let index = filter.value.as_int().unwrap_or(0) as u8;
				match filter.name.as_str() {
					"連載狀態" => filter_status_index = index,
					"內容分級" => filter_content_rating = index,
					_ => continue,
				}
			}

			FilterType::Genre => {
				let is_not_checked = filter.value.as_int().unwrap_or(-1) != 1;
				if is_not_checked {
					continue;
				}

				let tag = filter.name;
				filter_tags_vec.push(tag);
			}

			FilterType::Sort => {
				let object = filter.value.as_object()?;
				sort_by = object.get("index").as_int().unwrap_or(1) as u8;
			}

			_ => continue,
		}
	}

	let filter_tags = match filter_tags_vec.is_empty() {
		true => "0".to_string(),
		false => filter_tags_vec.join("+"),
	};
	// 1-{}-{}-{}-{}-{}-{type}-{viewing_permission}
	// type=[1: Manga, 2: Novel]
	// Login cookie is required to view manga for VIP members
	// viewing_permission=[0: General, 1: VIP, 2: All]
	let filters_path = format!(
		"cate/tp/1-{}-{}-{}-{}-{}-1-2",
		encode_uri(filter_tags),
		FILTER_STATUS[filter_status_index as usize],
		sort_by,
		page,
		filter_content_rating
	);
	url.push_str(filters_path.as_str());

	Ok(url)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{}{}{}{}", DOMAIN, HTML_PATH, MANGA_PATH, id);

	let manga_html = get(url.clone()).html()?;

	let cover = manga_html.select("a.play").attr("abs:data-original").read();

	let title = manga_html.select("div.title > h1").text().read();

	let mut artists_vec: Vec<String> = Vec::new();
	for value in manga_html.select("p.data:contains(作者：) > a").array() {
		let artist_str = value.as_node()?.text().read();
		artists_vec.push(artist_str);
	}
	let artist = artists_vec.join("、");

	let mut description = manga_html
		.select("span.detail-text")
		.html()
		.read()
		.replace("<br> ", "\n")
		.replace("<br>", "\n")
		.trim()
		.to_string();
	if let Some(description_with_closing_tag) = description.substring_before_last("</") {
		description = description_with_closing_tag.trim().to_string();
	}

	let mut categories: Vec<String> = Vec::new();
	let mut nsfw = MangaContentRating::Nsfw;
	for value in manga_html.select("a.tag > span").array() {
		let tag = value.as_node()?.text().read();

		if tag.is_empty() {
			continue;
		}

		if tag == "清水" {
			nsfw = MangaContentRating::Safe;
		}

		categories.push(tag);
	}

	let status = match manga_html.select("p.data").first().text().read().as_str() {
		"连载中" => MangaStatus::Ongoing,
		"完结" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};

	Ok(Manga {
		id,
		cover,
		title,
		author: artist.clone(),
		artist,
		description,
		url,
		categories,
		status,
		nsfw,
		..Default::default()
	})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let chapter_list_url = format!("{}{}chapter_list/tp/{}-0-0-10", DOMAIN, API_PATH, id);
	let chapter_list_json = get(chapter_list_url).json()?;
	let chapter_list_object = chapter_list_json.as_object()?;
	let result = chapter_list_object.get("result").as_object()?;

	let mut chapters: Vec<Chapter> = Vec::new();

	for (index, value) in result.get("list").as_array()?.rev().enumerate() {
		let manga_object = value.as_object()?;

		let id = manga_object.get("id").as_int()?.to_string();

		let title = manga_object.get("title").as_string()?.read();

		let chapter = (index + 1) as f32;

		let url = format!("{}{}{}{}", DOMAIN, HTML_PATH, CHAPTER_PATH, id);

		chapters.insert(
			0,
			Chapter {
				id,
				title,
				chapter,
				url,
				lang: "zh".to_string(),
				..Default::default()
			},
		);
	}

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let chapter_url = format!("{}{}{}{}", DOMAIN, HTML_PATH, CHAPTER_PATH, chapter_id);
	let chapter_html = get(chapter_url).html()?;

	let mut pages: Vec<Page> = Vec::new();

	for (index, value) in chapter_html.select("img.lazy[id]").array().enumerate() {
		let page_path = value
			.as_node()?
			.attr("data-original")
			.read()
			.trim()
			.to_string();
		let url = format!("{}{}", DOMAIN, page_path);

		pages.push(Page {
			index: index as i32,
			url,
			..Default::default()
		});
	}

	Ok(pages)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
		.header("Referer", DOMAIN)
		.header("User-Agent", USER_AGENT);
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	if !url.contains("/id/") {
		return Ok(DeepLink::default());
	}

	if url.contains(MANGA_PATH) {
		let Some(manga_id) = url.substring_after_last("/") else {
			return Ok(DeepLink::default());
		};
		let manga = Some(get_manga_details(manga_id.to_string())?);

		return Ok(DeepLink {
			manga,
			chapter: None,
		});
	}

	if !url.contains(CHAPTER_PATH) {
		return Ok(DeepLink::default());
	}

	let Some(chapter_id) = url.substring_after_last("/") else {
		return Ok(DeepLink::default());
	};
	let chapter = Some(Chapter {
		id: chapter_id.to_string(),
		..Default::default()
	});

	let chapter_html = get(url).html()?;
	let manga_url = chapter_html
		.select("a.icon-only.link.back")
		.attr("href")
		.read();
	let Some(manga_id) = manga_url.substring_after_last("/") else {
		return Ok(DeepLink { manga: None, chapter });
	};
	let manga = Some(get_manga_details(manga_id.to_string())?);

	Ok(DeepLink { manga, chapter })
}
