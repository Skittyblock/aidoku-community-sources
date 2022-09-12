#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{format, print, ObjectRef, String, StringRef, ValueRef, Vec},
	std::{
		json,
		net::{HttpMethod, Request},
	},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult,
	MangaStatus, MangaViewer, Page,
};
mod helper;

const FILTER_GENRE: [i32; 30] = [
	0, 31, 26, 1, 3, 27, 5, 2, 6, 8, 9, 25, 10, 11, 12, 17, 33, 37, 14, 15, 29, 20, 21, 4, 7, 30,
	34, 36, 40, 61,
];
const FILTER_STATUS: [i32; 3] = [0, 2, 1];
const FILTER_SORT: [i32; 3] = [10, 2, 18];

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

	print(format(format_args!("page: {}", page)));
	print(format(format_args!("query: {}", query)));
	print(format(format_args!("status: {}", status)));
	print(format(format_args!("genre: {}", genre)));
	print(format(format_args!("sort: {}", sort)));

	if query.is_empty() {
		return get_manga_list_by_filter(
			ListFilter {
				status,
				genre,
				sort,
			},
			page,
		);
	} else {
		return get_manga_list_by_query(query, page);
	}
}

#[get_manga_listing]
fn get_manga_listing(_: Listing, _: i32) -> Result<MangaPageResult> {
	print("get_manga_listing");

	todo!()
}

#[get_manga_details]
fn get_manga_details(_: String) -> Result<Manga> {
	todo!()
}

#[get_chapter_list]
fn get_chapter_list(_: String) -> Result<Vec<Chapter>> {
	todo!()
}

#[get_page_list]
fn get_page_list(_: String, _: String) -> Result<Vec<Page>> {
	todo!()
}

#[modify_image_request]
fn modify_image_request(_: Request) {}

#[handle_url]
fn handle_url(_: String) -> Result<DeepLink> {
	todo!()
}

fn get_manga_list_by_filter(filter: ListFilter, page: i32) -> Result<MangaPageResult> {
	const BASE_URL: &str = "https://www.manhuaren.com/manhua-list/dm5.ashx";

	let body_content = format!("action=getclasscomics&pageindex={}&pagesize=20&categoryid=0&tagid={}&status={}&usergroup=0&pay=-1&areaid=0&sort={}&iscopyright=0", page, filter.genre, filter.status, filter.sort);

	print(&body_content);

	let req = Request::new(BASE_URL, HttpMethod::Post)
		.body(body_content.as_bytes())
		.header("Content-Type", "application/x-www-form-urlencoded");

	let body = req.string()?;
	// print(&body);

	let json = json::parse(body)?.as_object()?;
	let result = json.get("UpdateComicItems").as_array()?;
	let item_count = result.len();

	print(format(format_args!("items: {}", item_count)));

	let mut manga_arr: Vec<Manga> = Vec::new();

	for manga in result {
		let manga_obj = manga.as_object()?;
		let mut author_vec: Vec<String> = Vec::new();
		let author_arr = manga_obj.get("Author").as_array()?;
		for a in author_arr {
			let s = a.as_string().unwrap_or(StringRef::from("-")).read();
			author_vec.push(s);
		}

		manga_arr.push(Manga {
			id: manga_obj
				.get("ID")
				.as_string()
				.unwrap_or(StringRef::from("-"))
				.read(),
			cover: manga_obj
				.get("ShowPicUrlB")
				.as_string()
				.unwrap_or(StringRef::from("-"))
				.read(),
			title: manga_obj
				.get("Title")
				.as_string()
				.unwrap_or(StringRef::from("-"))
				.read(),
			author: author_vec.join(", "),
			artist: author_vec.join(", "),
			description: manga_obj
				.get("Content")
				.as_string()
				.unwrap_or(StringRef::from("-"))
				.read(),
			url: String::from(""),
			categories: Vec::new(),
			status: match manga_obj.get("Status").as_int().unwrap_or(-1) {
				0 => MangaStatus::Ongoing,
				1 => MangaStatus::Completed,
				_ => MangaStatus::Unknown,
			},
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Vertical,
		});
	}

	return Ok(MangaPageResult {
		manga: manga_arr,
		has_more: item_count > 0,
	});
}

fn get_manga_list_by_query(query: String, page: i32) -> Result<MangaPageResult> {
	const BASE_URL: &str = "https://www.manhuaren.com/pagerdata.ashx";

	let body_content = format!(
		"t=7&pageindex={}&pagesize=20&f=0&title={}",
		page,
		helper::urlencode(query)
	);

	print(&body_content);

	let req = Request::new(BASE_URL, HttpMethod::Post)
		.body(body_content.as_bytes())
		.header("Content-Type", "application/x-www-form-urlencoded")
		.header("Referer", "https://www.manhuaren.com/manhua-list/");

	let body = req.string()?;
	// print(&body);

	let result = json::parse(body)?.as_array()?;
	let item_count = result.len();

	print(format(format_args!("items: {}", item_count)));

	let mut manga_arr: Vec<Manga> = Vec::new();

	for manga in result {
		let manga_obj = manga.as_object()?;
		let mut author_vec: Vec<String> = Vec::new();
		let author_arr = manga_obj.get("Author").as_array()?;
		for a in author_arr {
			let s = a.as_string().unwrap_or(StringRef::from("-")).read();
			author_vec.push(s);
		}

		manga_arr.push(Manga {
			id: manga_obj
				.get("Id")
				.as_string()
				.unwrap_or(StringRef::from("-"))
				.read(),
			cover: manga_obj
				.get("BigPic")
				.as_string()
				.unwrap_or(StringRef::from("-"))
				.read(),
			title: manga_obj
				.get("Title")
				.as_string()
				.unwrap_or(StringRef::from("-"))
				.read(),
			author: author_vec.join(", "),
			artist: author_vec.join(", "),
			description: manga_obj
				.get("Content")
				.as_string()
				.unwrap_or(StringRef::from("-"))
				.read(),
			url: String::from(""),
			categories: Vec::new(),
			status: match manga_obj
				.get("Status")
				.as_string()
				.unwrap_or(StringRef::from(""))
				.read()
				.as_str()
			{
				"连载中" => MangaStatus::Ongoing,
				"已完结" => MangaStatus::Completed,
				_ => MangaStatus::Unknown,
			},
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Vertical,
		});
	}

	return Ok(MangaPageResult {
		manga: manga_arr,
		has_more: false,
	});
}

struct ListFilter {
	status: i32,
	genre: i32,
	sort: i32,
}
