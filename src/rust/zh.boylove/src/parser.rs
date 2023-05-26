use aidoku::{
	error::Result,
	helpers::uri::{encode_uri, QueryParameters},
	prelude::{format, println},
	std::{html::Node, net::Request, String, ValueRef, Vec},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
};

extern crate alloc;
use alloc::string::ToString;

pub const BASE_URL: &str = "https://boylove.cc";
pub const API_URL: &str = "/home/api/";
pub const HTML_URL: &str = "/home/book/";
pub const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36";
const FILTER_STATUS: [u8; 3] = [2, 0, 1];

pub fn get_filtered_url(filters: Vec<Filter>, page: i32) -> String {
	let mut is_searching = false;

	let mut filter_status_index = 0;
	let mut filter_content_rating = 0;
	let mut filter_tag_vec: Vec<String> = Vec::new();
	let mut sort_by = 1;

	let mut query = QueryParameters::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(value) = filter.value.as_string() {
					is_searching = true;
					query.push("keyword", Some(value.read().as_str()));
					// type=[1: Manga, 2: Novel]
					query.push("type", Some("1"));
					query.push("pageNo", Some(page.to_string().as_str()));
				}
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
				let value = filter.value.as_int().unwrap_or(-1);
				if value < 1 {
					continue;
				}
				filter_tag_vec.push(filter.name)
			}
			FilterType::Sort => {
				if let Ok(value) = filter.value.as_object() {
					sort_by = value.get("index").as_int().unwrap_or(1) as u8;
				}
			}
			_ => continue,
		}
	}

	let mut url = format!("{}{}", BASE_URL, API_URL);
	if is_searching {
		url.push_str(format!("searchk?{}", query).as_str());
	} else {
		let filter_tag = if filter_tag_vec.is_empty() {
			"0".to_string()
		} else {
			filter_tag_vec.join("+")
		};
		// 1-{}-{}-{}-{}-{}-{type}-{viewing_permission}
		// type=[1: Manga, 2: Novel]
		// Login support is needed to view manga for VIP members
		// viewing_permission=[0: General, 1: VIP, 2: All]
		url.push_str(
			format!(
				"cate/tp/1-{}-{}-{}-{}-{}-1-2",
				encode_uri(filter_tag),
				FILTER_STATUS[filter_status_index as usize],
				sort_by,
				page,
				filter_content_rating
			)
			.as_str(),
		);
	}
	url
}

pub fn request_get(url: String) -> Request {
	Request::get(url)
		.header("Referer", BASE_URL)
		.header("User-Agent", USER_AGENT)
}

pub fn get_manga_list(json: ValueRef) -> Result<MangaPageResult> {
	let object = json.as_object()?;
	let result = object.get("result").as_object()?;

	let mut manga: Vec<Manga> = Vec::new();

	for item in result.get("list").as_array()? {
		let manga_item = item.as_object()?;

		// if manga_item.get("lanmu_id").as_int().unwrap_or(0) == 5 {
		// 	continue;
		// }
		// There's an ad whose lanmu_id is not 5
		let keywords = manga_item.get("keyword").as_string()?.read();
		if keywords.contains("公告") {
			continue;
		}

		let id = manga_item.get("id").as_int()?.to_string();
		let cover = format!(
			"{}{}",
			BASE_URL,
			manga_item.get("image").as_string()?.read()
		);
		let title = manga_item.get("title").as_string()?.read();
		let artist = manga_item
			.get("auther")
			.as_string()?
			.read()
			.replace('&', "、");
		let description = manga_item.get("desc").as_string()?.read();
		let url = format!("{}{}index/id/{}", BASE_URL, HTML_URL, id);
		let categories: Vec<String> = keywords.split(',').map(|tag| tag.to_string()).collect();
		let status = match manga_item.get("mhstatus").as_int()? {
			0 => MangaStatus::Ongoing,
			1 => MangaStatus::Completed,
			_ => MangaStatus::Unknown,
		};
		let nsfw = if categories.contains(&"清水".to_string()) {
			MangaContentRating::Safe
		} else {
			MangaContentRating::Nsfw
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

pub fn get_manga_details(html: Node, id: String) -> Result<Manga> {
	let cover = html.select("a.play").attr("abs:data-original").read();
	let title = html.select("div.title > h1").text().read();

	let mut artist_vec: Vec<String> = Vec::new();
	for item in html.select("p.data:contains(作者：) > a").array() {
		let artist_str = item.as_node()?.text().read();
		artist_vec.push(artist_str);
	}
	let artist = artist_vec.join("、");

	let description = html
		.select("span.detail-text")
		.html()
		.read()
		.replace("<br> ", "\n")
		.replace("<br>", "\n");
	let url = format!("{}{}index/id/{}", BASE_URL, HTML_URL, id);

	let mut categories: Vec<String> = Vec::new();
	for item in html.select("a.tag > span").array() {
		let tag = item.as_node()?.text().read();
		categories.push(tag);
	}

	let status = match html.select("p.data").first().text().read().as_str() {
		"连载中" => MangaStatus::Ongoing,
		"完结" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};
	let nsfw = if categories.contains(&"清水".to_string()) {
		MangaContentRating::Safe
	} else {
		MangaContentRating::Nsfw
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

pub fn get_chapter_list(json: ValueRef) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	let object = json.as_object()?;
	let result = object.get("result").as_object()?;

	for (index, item) in result.get("list").as_array()?.rev().enumerate() {
		let manga_item = item.as_object()?;

		let id = manga_item.get("id").as_int()?.to_string();
		let title = manga_item.get("title").as_string()?.read();
		let chapter = (index + 1) as f32;
		let url = format!("{}{}capter/id/{}", BASE_URL, HTML_URL, id);

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
