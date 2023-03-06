/*	Created by reference to https://github.com/tachiyomiorg/tachiyomi-extensions/tree/master/src/zh/dmzj
 *	All credit goes to their outstanding work.
 */

#![no_std]
use core::ops::Deref;

use aidoku::{
	error::Result,
	prelude::*,
	std::net::{HttpMethod, Request},
	std::{json, ArrayRef, String, Vec},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

mod helper;

const BASE_URL: &str = "https://m.dmzj.com";
const V3_API_URL: &str = "https://v3api.dmzj.com";
const V4_API_URL: &str = "https://nnv4api.dmzj.com";
const API_URL: &str = "https://api.dmzj.com";
const API_PAGELIST_OLD_URL: &str = "https://api.m.dmzj.com";
const API_PAGELIST_WEBVIEW_URL: &str = "https://m.dmzj.com/chapinfo";
// const IMAGE_URL: &str = "https://images.dmzj.com";
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
				sort = 1 - value.get("index").as_int()? as i32;
			}
			_ => continue,
		}
	}

	let mut manga_arr: Vec<Manga> = Vec::new();

	if is_keyword {
		let url = format!(
			"http://s.acg.dmzj.com/comicsum/search.php?s={}",
			&helper::encode_uri(&keyword)
		);

		// API return 404 randomly, try multi times.
		let mut index = 0;
		let data: ArrayRef = loop {
			if index == 8 {
				break ArrayRef::new();
			}

			let req = helper::get(&url);
			let r = req.string().unwrap();
			let rr = r.strip_prefix("var g_search_data = ");

			match rr {
				Some(rr) => {
					break json::parse(rr.strip_suffix(';').unwrap().as_bytes())?.as_array()?
				}
				_ => index += 1,
			}
		};

		for it in data {
			let it = it.as_object()?;

			manga_arr.push(Manga {
				id: helper::i32_to_string(it.get("id").as_int()? as i32),
				cover: it.get("comic_cover").as_string()?.read(),
				title: it.get("comic_name").as_string()?.read(),
				author: it
					.get("comic_author")
					.as_string()?
					.read()
					.replace('/', ", "),
				artist: String::new(),
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
			// Pop extra '-'
			filters_query.pop();
		}

		let url = format!(
			"{}/classify/{}/{}/{}.json",
			V3_API_URL,
			filters_query,
			helper::i32_to_string(sort),
			helper::i32_to_string(page)
		);
		let data = Request::new(&url, HttpMethod::Get).json()?.as_array()?;

		for it in data {
			let it = it.as_object()?;
			manga_arr.push(Manga {
				id: helper::i32_to_string(it.get("id").as_int()? as i32),
				cover: it.get("cover").as_string()?.read(),
				title: it.get("title").as_string()?.read(),
				// Nullable?, Meet once. Maybe api buggy.
				author: match it.get("authors").as_string() {
					Ok(authors) => authors.read().replace('/', ", "),
					Err(_) => String::new(),
				},
				artist: String::new(),
				description: String::new(),
				url: String::new(),
				categories: it
					.get("types")
					.as_string()?
					.read()
					.split('/')
					.collect::<Vec<_>>()
					.iter()
					.map(|s| String::from(s.deref()))
					.collect(),
				status: match it.get("status").as_string()?.read().as_str() {
					"连载中" => MangaStatus::Ongoing,
					"已完结" => MangaStatus::Completed,
					_ => MangaStatus::Unknown,
				},
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
	let url = format!("{}/comic/detail/{}?uid=2665531", V4_API_URL, &id);

	let pb = helper::decode_as_comic_detail(&helper::get(&url).string()?).unwrap();
	if pb.errno == 0 {
		let pb_data = pb.data.unwrap();
		return Ok(Manga {
			id: id.clone(),
			cover: pb_data.cover,
			title: pb_data.title,
			author: pb_data
				.authors
				.iter()
				.map(|s| s.tag_name.clone())
				.collect::<Vec<String>>()
				.join(", "),
			artist: String::new(),
			description: pb_data.description,
			url: format!("{}/info/{}.html", BASE_URL, id),
			categories: pb_data.types.iter().map(|s| s.tag_name.clone()).collect(),
			status: match pb_data.status[0].tag_name.as_str() {
				"连载中" => MangaStatus::Ongoing,
				"已完结" => MangaStatus::Completed,
				_ => MangaStatus::Unknown,
			},
			nsfw: MangaContentRating::Safe,
			viewer: match pb_data.direction {
				0 => MangaViewer::Rtl, // Maybe? Can't find evidence.
				1 => MangaViewer::Ltr,
				2 => MangaViewer::Scroll,
				_ => MangaViewer::Rtl,
			},
		});
	} else {
		// Try V3 api

		let url = format!("{}/dynamic/comicinfo/{}.json", API_URL, id);

		let json = helper::get(&url).json()?.as_object()?;

		let data = json.get("data").as_object()?;
		let info = data.get("info").as_object()?;
		let types = info.get("types").as_string()?.read();

		return Ok(Manga {
			id: id.clone(),
			cover: info.get("cover").as_string()?.read(),
			title: info.get("title").as_string()?.read(),
			author: info.get("authors").as_string()?.read().replace('/', ", "),
			artist: String::new(),
			description: info.get("description").as_string()?.read(),
			url: format!("{}/info/{}.html", BASE_URL, id),
			categories: types
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
			viewer: match info.get("direction").as_int()? {
				0 => MangaViewer::Rtl, // Maybe? Can't find evidence.
				1 => MangaViewer::Ltr,
				2 => MangaViewer::Scroll,
				_ => MangaViewer::Rtl,
			},
		});
	}
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	// Try V4 API first

	let url = format!("{}/comic/detail/{}?uid=2665531", V4_API_URL, &id);

	let pb = helper::decode_as_comic_detail(&helper::get(&url).string()?).unwrap();

	let mut chapters = Vec::new();
	if pb.errno == 0 && !pb.data.as_ref().unwrap().chapters.is_empty() {
		let pb_data = pb.data.unwrap();
		let mut volume = 0;
		let has_multi_chapter = pb_data.chapters.len() >= 2;
		for chapter_list in pb_data.chapters {
			volume += 1;
			let len = chapter_list.data.len();
			for (index, chapter) in chapter_list.data.into_iter().enumerate() {
				chapters.push(Chapter {
					id: format!("{}/{}", pb_data.id, chapter.chapter_id),
					title: format!("{}: {}", chapter_list.title, chapter.chapter_title),
					volume: if has_multi_chapter {
						volume as f32
					} else {
						-1.0
					},
					chapter: (len - index) as f32,
					date_updated: chapter.updatetime as f64,
					scanlator: String::new(),
					url: String::new(),
					lang: String::from("zh"),
				});
			}
		}
	} else {
		// Try V3 API

		let url = format!("{}/dynamic/comicinfo/{}.json", API_URL, id);
		let json = helper::get(&url).json()?.as_object()?;
		let data = json.get("data").as_object()?;
		let list = data.get("list").as_array()?;

		let len = list.len();

		for (index, chapter) in list.enumerate() {
			let data = chapter.as_object()?;

			chapters.push(Chapter {
				id: format!("{}/{}", id, data.get("id").as_string()?.read()),
				title: data.get("chapter_name").as_string()?.read(),
				volume: -1.0,
				chapter: (len - index) as f32,
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
fn get_page_list(_manga_id: String, id: String) -> Result<Vec<Page>> {
	// Not Tested
	// Maybe only use the first one.
	let url = [
		format!("{}/{}.html", API_PAGELIST_WEBVIEW_URL, &id),
		format!(
			"{}/chapter/{}.json?channel=android&version=3.0.0&timestamp={}",
			API_URL,
			&id,
			aidoku::std::current_date() as i64
		),
		format!("{}/comic/chapter/{}.html", API_PAGELIST_OLD_URL, &id),
	];
	let mut index = 0;
	let arr: Vec<String> = loop {
		if index >= url.len() {
			break Vec::new();
		}

		let req = helper::get(&url[index]).json();

		let r = match index {
			0 | 1 => req?.as_object()?.get("page_url").clone().as_array().ok(),
			2 => req?
				.as_object()?
				.get("chapter")
				.as_object()?
				.get("page_url")
				// STILL NOT FIXED
				.clone()
				// DO NOT REMOVE
				.as_array()
				.ok(),
			_ => None,
		};
		match r {
			Some(r) => {
				// Check if image url valid by having an extension.
				let mut rr: Vec<String> = Vec::new();
				for it in r {
					let str = it.as_string()?.read();

					if let Some(mat) = str.rfind('.') {
						match str[mat..str.len()].to_lowercase().as_str() {
							".jpg" | ".png" | ".gif" => rr.push(str),
							_ => {}
						}
					}
				}
				break rr;
			}
			_ => index += 1,
		};
	};

	// Image fallback
	let thumb = {
		if !arr.is_empty() {
			let r = Request::get(&helper::encode_uri(&arr[0]));
			r.send();
			!matches!(r.status_code(), 200)
		} else {
			false
		}
	};

	let mut pages = Vec::new();

	for (index, r) in arr.iter().enumerate() {
		let mut image_url = String::from(r.deref());
		image_url = image_url
			.replace("http:", "https:")
			.replace("dmzj1.com", "dmzj.com");

		let thumb_url = {
			if !id.is_empty() {
				let initial = image_url
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
			url: helper::encode_uri(match thumb {
				true => &thumb_url,
				false => &image_url,
			}),
			base64: String::new(),
			text: String::new(),
		});
	}

	Ok(pages)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
    .header("Referer", "https://www.dmzj.com/")
    .header("User-Agent",
    "Mozilla/5.0 (Linux; Android 10) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.93 Mobile Safari/537.36 Aidoku/1.0");
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
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
