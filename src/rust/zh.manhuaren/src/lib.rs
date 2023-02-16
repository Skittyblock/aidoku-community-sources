#![no_std]
extern crate alloc;
use aidoku::{
	error::Result,
	helpers::uri::encode_uri,
	prelude::*,
	std::{json, net::HttpMethod, net::Request},
	std::{ObjectRef, String, Vec},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

use alloc::{string::ToString, vec};
mod helper;

const FILTER_GENRE: [i32; 30] = [
	0, 31, 26, 1, 3, 27, 5, 2, 6, 8, 9, 25, 10, 11, 12, 17, 33, 37, 14, 15, 29, 20, 21, 4, 7, 30,
	34, 36, 40, 61,
];
const FILTER_STATUS: [i32; 3] = [0, 1, 2];
const FILTER_SORT: [i32; 3] = [0, 1, 2];

const API_URL: &str = "http://mangaapi.manhuaren.com";

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
		.header("X-Yq-Yqci", "{\"le\": \"zh\"}")
		.header("User-Agent", "okhttp/3.11.0")
		.header("Referer", "http://www.dm5.com/dm5api/")
		.header("clubReferer", "http://mangaapi.manhuaren.com/");
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();
	let mut status: i32 = 0;
	let mut genre: i32 = 0;
	let mut sort: i32 = 10;

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				let index = filter.value.as_int()? as usize;
				match filter.name.as_str() {
					"连载状态" => {
						status = FILTER_STATUS[index];
					}
					"分类" => {
						genre = FILTER_GENRE[index];
					}
					_ => continue,
				};
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int()? as usize;
				sort = FILTER_SORT[index];
			}
			_ => continue,
		}
	}

	if query.is_empty() {
		get_manga_list_by_filter(
			ListFilter {
				status,
				genre,
				sort,
			},
			page,
		)
	} else {
		get_manga_list_by_query(query, page)
	}
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let mut args: Vec<(String, String)> = Vec::new();
	args.push((String::from("mangaId"), id.clone()));

	let qs = helper::generate_get_query(&mut args);

	let url = String::from(API_URL) + "/v1/manga/getDetail?" + &qs;

	let body = helper::request(url, HttpMethod::Get)?;
	let json = json::parse(body)?.as_object()?;
	let manga = json.get("response").as_object()?;

	let category_str = helper::stringref_unwrap_or_empty(manga.get("mangaTheme").as_string());

	let categories: Vec<String> = match category_str.is_empty() {
		true => Vec::new(),
		false => category_str
			.split(' ')
			.collect::<Vec<_>>()
			.iter()
			.map(|c| String::from(*c))
			.collect(),
	};

	let cover = helper::stringref_unwrap_or_fallback(
		manga.get("mangaPicimageUrl").as_string(),
		helper::stringref_unwrap_or_empty(manga.get("shareIcon").as_string()),
	);

	Ok(Manga {
		id: match manga.get("mangaId").as_int() {
			Ok(str) => str.to_string(),
			Err(_) => id,
		},
		cover,
		title: helper::stringref_unwrap_or_empty(manga.get("mangaName").as_string()),
		author: helper::stringref_unwrap_or_empty(manga.get("mangaAuthor").as_string()),
		artist: helper::stringref_unwrap_or_empty(manga.get("mangaAuthor").as_string()),
		description: helper::stringref_unwrap_or_empty(manga.get("mangaIntro").as_string()),
		url: helper::stringref_unwrap_or_empty(manga.get("shareUrl").as_string()),
		categories,
		status: match manga.get("mangaIsOver").as_int().unwrap_or(-1) {
			0 => MangaStatus::Ongoing,
			1 => MangaStatus::Completed,
			_ => MangaStatus::Unknown,
		},
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Vertical,
	})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let mut args: Vec<(String, String)> = Vec::new();

	args.push((String::from("mangaId"), id));

	let qs = helper::generate_get_query(&mut args);

	let url = String::from(API_URL) + "/v1/manga/getDetail?" + &qs;
	let body = helper::request(url, HttpMethod::Get)?;

	let json = json::parse(body)?.as_object()?;
	let manga = json.get("response").as_object()?;

	let mut chapter_arr: Vec<Chapter> = Vec::new();

	chapter_arr.append(&mut parse_chapters(&manga, "mangaEpisode"));
	chapter_arr.append(&mut parse_chapters(&manga, "mangaWords"));
	chapter_arr.append(&mut parse_chapters(&manga, "mangaRolls"));

	Ok(chapter_arr)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut args: Vec<(String, String)> = vec![
		(String::from("mangaId"), manga_id),
		(String::from("mangaSectionId"), chapter_id),
		(String::from("netType"), String::from("3")),
		(String::from("loadreal"), String::from("1")),
		(String::from("imageQuality"), String::from("2")),
	];

	let qs = helper::generate_get_query(&mut args);

	let url = String::from(API_URL) + "/v1/manga/getRead?" + &qs;

	let body = helper::request(url, HttpMethod::Get)?;

	let json = json::parse(body)?.as_object()?;
	let manga = json.get("response").as_object()?;

	Ok(parse_page(&manga))
}

fn get_manga_list_by_filter(filter: ListFilter, page: i32) -> Result<MangaPageResult> {
	let mut args: Vec<(String, String)> = vec![
		(String::from("subCategoryType"), String::from("0")),
		(String::from("subCategoryId"), filter.genre.to_string()),
		(String::from("start"), ((page - 1) * 20).to_string()),
		(String::from("limit"), String::from("20")),
		(String::from("sort"), filter.sort.to_string()),
		(String::from("status"), filter.status.to_string()),
	];

	let qs = helper::generate_get_query(&mut args);

	let url = String::from(API_URL) + "/v2/manga/getCategoryMangas?" + &qs;
	let body = helper::request(url, HttpMethod::Get)?;
	let json = json::parse(body)?.as_object()?;
	let response = json.get("response").as_object()?;
	let mangas = response.get("mangas").as_array()?;
	let item_count = mangas.len();

	let mut manga_arr: Vec<Manga> = Vec::new();

	for manga in mangas {
		let manga_obj = manga.as_object()?;

		let cover = helper::stringref_unwrap_or_fallback(
			manga_obj.get("mangaPicimageUrl").as_string(),
			helper::stringref_unwrap_or_empty(manga_obj.get("mangaCoverimageUrl").as_string()),
		);

		manga_arr.push(Manga {
			id: manga_obj
				.get("mangaId")
				.as_int()
				.unwrap_or_default()
				.to_string(),
			cover,
			title: helper::stringref_unwrap_or_empty(manga_obj.get("mangaName").as_string()),
			author: helper::stringref_unwrap_or_empty(manga_obj.get("mangaAuthor").as_string()),
			artist: helper::stringref_unwrap_or_empty(manga_obj.get("mangaAuthor").as_string()),
			status: match manga_obj.get("mangaIsOver").as_int().unwrap_or(-1) {
				0 => MangaStatus::Ongoing,
				1 => MangaStatus::Completed,
				_ => MangaStatus::Unknown,
			},
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Vertical,
			..Default::default()
		});
	}

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: item_count > 0,
	})
}

fn get_manga_list_by_query(query: String, page: i32) -> Result<MangaPageResult> {
	let mut args: Vec<(String, String)> = vec![
		(String::from("keywords"), query),
		(String::from("start"), ((page - 1) * 20).to_string()),
		(String::from("limit"), String::from("20")),
	];

	let qs = helper::generate_get_query(&mut args);

	let url = String::from(API_URL) + "/v1/search/getSearchManga?" + &qs;

	let body = helper::request(url, HttpMethod::Get)?;

	let json = json::parse(body)?.as_object()?;
	let response = json.get("response").as_object()?;
	let mangas = response.get("result").as_array()?;
	let item_count = mangas.len();

	let mut manga_arr: Vec<Manga> = Vec::new();

	for manga in mangas {
		let manga_obj = manga.as_object()?;

		manga_arr.push(Manga {
			id: manga_obj.get("mangaId").as_int().unwrap_or_default().to_string(),
			cover:
				// api won't return mangaPicimageUrl, no need to check
				helper::stringref_unwrap_or_empty(manga_obj
					.get("mangaCoverimageUrl")
					.as_string()),
			title: helper::stringref_unwrap_or_empty(manga_obj
				.get("mangaName")
				.as_string()),
			author: helper::stringref_unwrap_or_empty(manga_obj
				.get("mangaAuthor")
				.as_string()),
			artist: helper::stringref_unwrap_or_empty(manga_obj
				.get("mangaAuthor")
				.as_string()),
			status: match manga_obj.get("mangaIsOver").as_int().unwrap_or(-1) {
				0 => MangaStatus::Ongoing,
				1 => MangaStatus::Completed,
				_ => MangaStatus::Unknown,
			},
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Vertical,
			..Default::default()
		});
	}

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: item_count > 0,
	})
}

fn parse_chapters(manga: &ObjectRef, key: &str) -> Vec<Chapter> {
	match manga.get(key).as_array() {
		Ok(chapters) => {
			let mut chapter_arr: Vec<Chapter> = Vec::new();

			for ch in chapters {
				let ch_obj = match ch.as_object() {
					Ok(obj) => obj,
					Err(_) => continue,
				};

				let mut title = String::new();

				if ch_obj.get("isMustPay").as_int().unwrap_or_default() == 1 {
					title.push_str("(锁) ");
				}

				if key.eq("mangaEpisode") {
					title.push_str("[番外] ");
				}

				let section_name =
					helper::stringref_unwrap_or_empty(ch_obj.get("sectionName").as_string());
				let section_title =
					helper::stringref_unwrap_or_empty(ch_obj.get("sectionTitle").as_string());

				title.push_str(&section_name);

				if !section_title.is_empty() {
					title.push_str(": ");
					title.push_str(&section_title);
				}

				chapter_arr.push(Chapter {
					id: ch_obj
						.get("sectionId")
						.as_int()
						.unwrap_or_default()
						.to_string(),
					title,
					volume: -1.0,
					chapter: ch_obj.get("sectionSort").as_float().unwrap_or_default() as f32,
					date_updated: match ch_obj.get("releaseTime").as_date(
						"yyyy-MM-dd",
						Option::from("zh"),
						Option::from("TW"),
					) {
						Ok(d) => d,
						_ => -1.0,
					},
					lang: String::from("zh"),
					..Default::default()
				})
			}

			chapter_arr
		}
		_ => Vec::new(),
	}
}

fn parse_page(chapter: &ObjectRef) -> Vec<Page> {
	match chapter.get("mangaSectionImages").as_array() {
		Ok(pages) => {
			let host_list = chapter.get("hostList").as_array().expect("hostList Error");
			let host = helper::stringref_unwrap_or_empty(host_list.get(0).as_string());
			let query = helper::stringref_unwrap_or_empty(chapter.get("query").as_string());

			let mut page_arr: Vec<Page> = Vec::new();

			for (i, p) in pages.enumerate() {
				let p_str = helper::stringref_unwrap_or_empty(p.as_string());

				let mut url = encode_uri(String::from(&host));
				url.push_str(&p_str);
				url.push_str(&query);

				page_arr.push(Page {
					index: (i + 1) as i32,
					url,
					..Default::default()
				});
			}
			page_arr
		}
		Err(_) => Vec::new(),
	}
}

struct ListFilter {
	status: i32,
	genre: i32,
	sort: i32,
}
