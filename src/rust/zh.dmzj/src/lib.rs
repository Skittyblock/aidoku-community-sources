/*	Created by reference to https://github.com/tachiyomiorg/tachiyomi-extensions/tree/master/src/zh/dmzj
 *	All credit goes to their outstanding work.
 */

#![no_std]
use core::ops::Deref;

use aidoku::{
	error::Result,
	prelude::*,
	std::net::HttpMethod,
	std::net::Request,
	std::{json, ArrayRef},
	std::{String, Vec},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

mod helper;

const BASE_URL: &str = "https://m.dmzj.com";
const V3_API_URL: &str = "https://v3api.dmzj.com";
const V3_API_CHAPTER_URL: &str = "https://nnv3api.muwai.com";
// v3api now shutdown the functionality to fetch manga detail and chapter list,
// so move these logic to v4api
const V4_API_URL: &str = "https://nnv4api.muwai.com"; // https://v4api.dmzj1.com
const API_URL: &str = "https://api.dmzj.com";
const API_PAGELIST_OLD_URL: &str = "https://api.m.dmzj.com";
const API_PAGELIST_WEBVIEW_URL: &str = "https://m.dmzj.com/chapinfo";
const IMAGE_URL: &str = "https://images.dmzj.com";
const IMAGE_SMALL_URL: &str = "https://imgsmall.dmzj.com";

const FILTER_GENRE: [i32; 42] = [
	0, 4, 3243, 3242, 17, 3244, 3245, 3249, 3248, 3246, 16, 14, 7, 6, 5, 8, 9, 13, 12, 11, 10,
	3250, 3251, 5806, 5345, 5077, 5848, 6316, 7900, 7568, 6437, 4518, 4459, 3254, 3253, 3252, 3255,
	6219, 3328, 3365, 3326, 3325,
];

const FILTER_STATUS: [i32; 3] = [0, 2309, 2310];
const FILTER_READER: [i32; 4] = [0, 3262, 3263, 3264];
const FILTER_TYPE: [i32; 7] = [0, 2304, 2305, 2306, 2307, 2308, 8453];

#[get_manga_list]
pub fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut is_keyword: bool = false;
	let mut keyword: String = String::new();
	let mut sort: i32 = 0;
	let mut filters_list: Vec<i32> = Vec::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				let title = filter.value.as_string()?.read();
				is_keyword = !title.is_empty();

				if is_keyword {
					keyword = title;
					break;
				}
			}
			FilterType::Select => {
				let index = filter.value.as_int()? as usize;
				let id = match filter.name.as_str() {
					"连载状态" => FILTER_STATUS[index],
					"读者" => FILTER_READER[index],
					"地区" => FILTER_TYPE[index],
					"分类" => FILTER_GENRE[index],
					_ => continue,
				};

				if id != 0 {
					filters_list.push(id);
				}
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				sort = value.get("index").as_int()? as i32;
			}
			_ => continue,
		}
	}

	let mut manga_arr: Vec<Manga> = Vec::new();

	if is_keyword {
		let url = format!(
			"http://s.acg.dmzj.com/comicsum/search.php?s={}",
			&helper::encodeURI(&keyword)
		);

		let data = {
			let req = helper::GET(url);
			let r = req.string();

			let r = r
				.strip_prefix("var g_search_data = ")
				.unwrap()
				.strip_suffix(';')
				.unwrap();

			json::parse(r.as_bytes()).as_array()?
		};

		for it in data {
			let it = it.as_object()?;
			let id = helper::i32_to_string(it.get("id").as_int()? as i32);
			let author = it.get("comic_author").as_string()?.read();
			let title = it.get("comic_name").as_string()?.read();
			let cover = it.get("comic_cover").as_string()?.read();

			manga_arr.push(Manga {
				id,
				cover,
				title,
				author: String::new(),
				artist: author,
				description: String::new(),
				url: String::new(),
				categories: Vec::new(),
				status: MangaStatus::Unknown,
				nsfw: MangaContentRating::Safe,
				viewer: MangaViewer::Rtl,
			});
		}
	} else {
		let mut filters_query: String = String::from("0");
		if !filters_list.is_empty() {
			for i in filters_list {
				filters_query.push_str(&helper::i32_to_string(i));
				filters_query.push('-');
			}
			filters_query.pop();
		}

		let url = format!(
			"{}/classify/{}/{}/{}.json",
			V3_API_URL,
			filters_query,
			helper::i32_to_string(sort),
			helper::i32_to_string(page)
		);

		let data = Request::new(&url, HttpMethod::Get).json().as_array()?;

		for it in data {
			let it = it.as_object()?;
			let id = helper::i32_to_string(it.get("id").as_int()? as i32);
			let author = it.get("authors").as_string()?.read();
			let title = it.get("title").as_string()?.read();
			let cover = it.get("cover").as_string()?.read();
			let status = match it.get("status").as_string()?.read().as_str() {
				"连载中" => MangaStatus::Ongoing,
				"已完结" => MangaStatus::Completed,
				_ => MangaStatus::Unknown,
			};
			manga_arr.push(Manga {
				id,
				cover,
				title,
				author: String::new(),
				artist: author,
				description: String::new(),
				url: String::new(),
				categories: Vec::new(),
				status,
				nsfw: MangaContentRating::Safe,
				viewer: MangaViewer::Rtl,
			});
		}
	}

	let len = manga_arr.len();
	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: if is_keyword { false } else { len != 0 },
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!(
		"{}/comic/detail/{}?channel=android&version=3.0.0&timestamp={}",
		V4_API_URL,
		&id,
		aidoku::std::current_date() as i64
	);

	let pb = helper::DECODE(&helper::GET(url).string());
	if pb.errno == 0 {
		let pbData = pb.data.unwrap();
		return Ok(Manga {
			id: id.clone(),
			cover: pbData.cover,
			title: pbData.title,
			author: String::new(),
			artist: String::new(),
			description: pbData.description,
			url: format!("{}/info/{}.html", BASE_URL, id),
			categories: pbData.types.iter().map(|s| s.tag_name.clone()).collect(),
			status: match pbData.status[0].tag_name.as_str() {
				"连载中" => MangaStatus::Ongoing,
				"已完结" => MangaStatus::Completed,
				_ => MangaStatus::Unknown,
			},
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Ltr,
		});
	} else {
		// Try old api

		let url = format!("{}/dynamic/comicinfo/{}.json", API_URL, id);
		let req = helper::GET(url);

		let info = req
			.json()
			.as_object()?
			.get("data")
			.as_object()?
			.get("info")
			.as_object()?;

		return Ok(Manga {
			id: id.clone(),
			cover: info.get("cover").as_string()?.read(),
			title: info.get("title").as_string()?.read(),
			author: info.get("authors").as_string()?.read(),
			artist: String::from(""),
			description: info.get("description").as_string()?.read(),
			url: format!("{}/info/{}.html", BASE_URL, id),
			categories: info
				.get("types")
				.as_string()?
				.read()
				.split('/')
				.collect::<Vec<_>>()
				.iter()
				.map(|s| String::from(s.deref()))
				.collect(),
			status: match info.get("status").as_string()?.read().as_str() {
				"连载中" => MangaStatus::Ongoing,
				"已完结" => MangaStatus::Completed,
				_ => MangaStatus::Unknown,
			},
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Ltr,
		});
	}
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!(
		"{}/comic/detail/{}?channel=android&version=3.0.0&timestamp={}",
		V4_API_URL,
		&id,
		aidoku::std::current_date() as i64
	);

	let pb = helper::DECODE(&helper::GET(url).string());

	let mut chapters = Vec::new();

	if pb.errno == 0 && !pb.data.as_ref().unwrap().chapters.is_empty() {
		let pbData = pb.data.unwrap();
		let mut volume = 0;
		let hasMultiChapter = pbData.chapters.len() >= 2;
		for chapterList in pbData.chapters {
			volume += 1;
			for chapter in chapterList.data {
				chapters.push(Chapter {
					id: format!("{}/{}", pbData.id, chapter.chapter_id),
					title: format!("{}: {}", chapterList.title, chapter.chapter_title),
					volume: if hasMultiChapter { volume as f32 } else { -1.0 },
					chapter: (chapter.chapter_order / 10) as f32,
					date_updated: chapter.updatetime as f64,
					scanlator: String::new(),
					url: String::new(),
					lang: String::from("zh"),
				});
			}
		}
	} else {
		let url = format!("{}/dynamic/comicinfo/{}.json", API_URL, id);
		println!("{}", url);
		let req = helper::GET(url);

		let list = req
			.json()
			.as_object()?
			.get("data")
			.as_object()?
			.get("list")
			.clone()
			.as_array()?;

		for chapter in list {
			let data = chapter.as_object()?;

			chapters.push(Chapter {
				id: format!("{}/{}", id.clone(), data.get("id").as_string()?.read()),
				title: data.get("chapter_name").as_string()?.read(),
				volume: -1.0,
				chapter: (data.get("chapter_order").as_int()? / 10) as f32,
				date_updated: data.get("updatetime").as_int()? as f64,
				scanlator: String::new(),
				url: String::new(),
				lang: String::from("zh"),
			});
		}
	}
	Ok(chapters)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	// Not Tested
	// Maybe only use the first one.

	let url = [
		format!("{}/{}.html", API_PAGELIST_WEBVIEW_URL, &id),
		format!(
			"{}/chapter/{}.json?channel=android&version=3.0.0&timestamp={}",
			V3_API_CHAPTER_URL,
			&id,
			aidoku::std::current_date() as i64
		),
		format!("{}/comic/chapter/{}.html", API_PAGELIST_OLD_URL, &id),
	];
	let mut index = 0;

	let arr = loop {
		if index > 2 {
			break ArrayRef::new();
		}

		let req = helper::GET(url[index].clone());

		let req = req.json();
		let r = match index {
			0 | 1 => req.as_object()?.get("page_url").clone().as_array().ok(),
			2 => req
				.as_object()?
				.get("chapter")
				.as_object()?
				.get("page_url")
				.clone()
				.as_array()
				.ok(),
			_ => None,
		};
		match r {
			Some(r) => break r,
			_ => index += 1,
		};
	};

	let mut pages = Vec::new();

	for (index, r) in arr.enumerate() {
		let mut imageUrl = r.as_string()?.read();
		imageUrl = imageUrl
			.replace("http:", "https:")
			.replace("dmzj1.com", "dmzj.com");

		let _thumbUrl = {
			if !id.is_empty() {
				let initial = imageUrl
					.strip_prefix("https://images.dmzj.com/")
					.unwrap()
					.get(0..1)
					.unwrap();

				format!("{}/{}/{}/{}.jpg", IMAGE_SMALL_URL, initial, id, index)
			} else {
				String::new()
			}
		};

		pages.push(Page {
			index: index as i32,
			url: helper::encodeURI(&imageUrl),
			base64: String::new(),
			text: String::new(),
		});
	}

	Ok(pages)
}

// Doesn't work
#[modify_image_request]
fn modify_image_request(request: Request) {
	request
    .header("Referer", "https://www.dmzj.com/")
    .header("User-Agent",
    "Mozilla/5.0 (Linux; Android 10) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.93 Mobile Safari/537.36 Aidoku/1.0");
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	println!("{}", url);

	let prefix = [
		"https://m.dmzj.com/info/",
		"https://www.dmzj.com/info/",
		"https://manhua.dmzj.com/",
	];

	let mut index = 0;
	let manga_id = loop {
		if index > 2 {
			break String::new();
		}

		let r = url.strip_prefix(prefix[index]);
		match r {
			Some(str) => break String::from(str.strip_suffix(".html").unwrap_or_default()),
			_ => index += 1,
		}
	};

	if !url.is_empty() && index <= 2 {
		let manga = get_manga_details(manga_id)?;

		Ok(DeepLink {
			manga: Some(manga),
			chapter: None,
		})
	} else {
		Err(aidoku::error::AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		})
	}
}
