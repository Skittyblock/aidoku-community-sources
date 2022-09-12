#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{format, print, ObjectRef, String, StringRef, Vec},
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
const FILTER_STATUS: [i32; 3] = [0, 1, 2];
const FILTER_SORT: [i32; 3] = [0, 1, 2];

const API_URL: &str = "http://mangaapi.manhuaren.com";

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
fn get_manga_details(id: String) -> Result<Manga> {
	let mut args: Vec<(String, String)> = Vec::new();

	args.push((String::from("mangaId"), id));

	let qs = helper::generate_get_query(&mut args);

	print("qs:");
	print(&qs);

	let url = String::from(API_URL) + "/v1/manga/getDetail?" + &qs;
	print("url:");
	print(&url);

	let req = Request::new(url, HttpMethod::Get);

	let body = req.string()?;
	// print(&body);

	let json = json::parse(body)?.as_object()?;
	let manga = json.get("response").as_object()?;

	let category_str = manga
		.get("mangaTheme")
		.as_string()
		.unwrap_or(StringRef::from(""))
		.read();

	let categories: Vec<&str> = category_str.split(" ").collect();

	return Ok(Manga {
		id: helper::i32_to_string(manga.get("mangaId").as_int().unwrap_or(0) as i32),
		cover: manga
			.get("mangaPicimageUrl")
			.as_string()
			.unwrap_or(StringRef::from("-"))
			.read(),
		title: manga
			.get("mangaName")
			.as_string()
			.unwrap_or(StringRef::from("-"))
			.read(),
		author: manga
			.get("mangaAuthor")
			.as_string()
			.unwrap_or(StringRef::from("-"))
			.read(),
		artist: manga
			.get("mangaAuthor")
			.as_string()
			.unwrap_or(StringRef::from("-"))
			.read(),
		description: manga
			.get("mangaIntro")
			.as_string()
			.unwrap_or(StringRef::from("-"))
			.read(),
		url: manga
			.get("shareUrl")
			.as_string()
			.unwrap_or(StringRef::from("-"))
			.read(),
		categories: categories.iter().map(|c| String::from(*c)).collect(),
		status: match manga.get("mangaIsOver").as_int().unwrap_or(-1) {
			0 => MangaStatus::Ongoing,
			1 => MangaStatus::Completed,
			_ => MangaStatus::Unknown,
		},
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Vertical,
	});
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let mut args: Vec<(String, String)> = Vec::new();

	args.push((String::from("mangaId"), id));

	let qs = helper::generate_get_query(&mut args);

	print("qs:");
	print(&qs);

	let url = String::from(API_URL) + "/v1/manga/getDetail?" + &qs;
	print("url:");
	print(&url);

	let req = Request::new(url, HttpMethod::Get);

	let body = req.string()?;
	// print(&body);

	let json = json::parse(body)?.as_object()?;
	let manga = json.get("response").as_object()?;

	let mut chapter_arr: Vec<Chapter> = Vec::new();

	chapter_arr.append(&mut parse_chapters(&manga, "mangaEpisode"));
	chapter_arr.append(&mut parse_chapters(&manga, "mangaWords"));
	chapter_arr.append(&mut parse_chapters(&manga, "mangaRolls"));

	return Ok(chapter_arr);
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	print("get_page_list:");
	print(&manga_id);
	print(&chapter_id);

	let mut args: Vec<(String, String)> = Vec::new();

	args.push((String::from("mangaId"), manga_id));
	args.push((String::from("mangaSectionId"), chapter_id));
	args.push((String::from("netType"), String::from("3")));
	args.push((String::from("loadreal"), String::from("1")));
	args.push((String::from("imageQuality"), String::from("2")));

	let qs = helper::generate_get_query(&mut args);

	print("qs:");
	print(&qs);

	let url = String::from(API_URL) + "/v1/manga/getRead?" + &qs;
	print("url:");
	print(&url);

	let req = Request::new(url, HttpMethod::Get);

	let body = req.string()?;
	// print(&body);

	let json = json::parse(body)?.as_object()?;
	let manga = json.get("response").as_object()?;

	return Ok(parse_page(&manga));
}

#[modify_image_request]
fn modify_image_request(_: Request) {}

#[handle_url]
fn handle_url(_: String) -> Result<DeepLink> {
	todo!()
}

fn get_manga_list_by_filter(filter: ListFilter, page: i32) -> Result<MangaPageResult> {
	let mut args: Vec<(String, String)> = Vec::new();

	args.push((String::from("subCategoryType"), String::from("0")));
	args.push((
		String::from("subCategoryId"),
		helper::i32_to_string(filter.genre),
	));
	args.push((
		String::from("start"),
		helper::i32_to_string((page - 1) * 20),
	));
	args.push((String::from("limit"), String::from("20")));
	args.push((String::from("sort"), helper::i32_to_string(filter.sort)));
	args.push((String::from("status"), helper::i32_to_string(filter.status)));

	let qs = helper::generate_get_query(&mut args);

	print("qs:");
	print(&qs);

	let url = String::from(API_URL) + "/v2/manga/getCategoryMangas?" + &qs;
	print("url:");
	print(&url);

	let req = Request::new(url, HttpMethod::Get);

	let body = req.string()?;
	// print(&body);

	let json = json::parse(body)?.as_object()?;
	let response = json.get("response").as_object()?;
	let mangas = response.get("mangas").as_array()?;
	let item_count = mangas.len();

	print(format(format_args!("items: {}", item_count)));

	let mut manga_arr: Vec<Manga> = Vec::new();

	for manga in mangas {
		let manga_obj = manga.as_object()?;

		manga_arr.push(Manga {
			id: helper::i32_to_string(manga_obj.get("mangaId").as_int().unwrap_or(0) as i32),
			cover: manga_obj
				.get("mangaPicimageUrl")
				.as_string()
				.unwrap_or(StringRef::from("-"))
				.read(),
			title: manga_obj
				.get("mangaName")
				.as_string()
				.unwrap_or(StringRef::from("-"))
				.read(),
			author: manga_obj
				.get("mangaAuthor")
				.as_string()
				.unwrap_or(StringRef::from("-"))
				.read(),
			artist: manga_obj
				.get("mangaAuthor")
				.as_string()
				.unwrap_or(StringRef::from("-"))
				.read(),
			description: String::from(""),
			url: String::from(""),
			categories: Vec::new(),
			status: match manga_obj.get("mangaIsOver").as_int().unwrap_or(-1) {
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
	let mut args: Vec<(String, String)> = Vec::new();

	args.push((String::from("keywords"), query));
	args.push((
		String::from("start"),
		helper::i32_to_string((page - 1) * 20),
	));
	args.push((String::from("limit"), String::from("20")));

	let qs = helper::generate_get_query(&mut args);

	print("qs:");
	print(&qs);

	let url = String::from(API_URL) + "/v1/search/getSearchManga?" + &qs;
	print("url:");
	print(&url);

	let req = Request::new(url, HttpMethod::Get);

	let body = req.string()?;
	// print(&body);

	let json = json::parse(body)?.as_object()?;
	let response = json.get("response").as_object()?;
	let mangas = response.get("result").as_array()?;
	let item_count = mangas.len();

	print(format(format_args!("items: {}", item_count)));

	let mut manga_arr: Vec<Manga> = Vec::new();

	for manga in mangas {
		let manga_obj = manga.as_object()?;

		manga_arr.push(Manga {
			id: helper::i32_to_string(manga_obj.get("mangaId").as_int().unwrap_or(0) as i32),
			cover: manga_obj
				.get("mangaCoverimageUrl")
				.as_string()
				.unwrap_or(StringRef::from("-"))
				.read(),
			title: manga_obj
				.get("mangaName")
				.as_string()
				.unwrap_or(StringRef::from("-"))
				.read(),
			author: manga_obj
				.get("mangaAuthor")
				.as_string()
				.unwrap_or(StringRef::from("-"))
				.read(),
			artist: manga_obj
				.get("mangaAuthor")
				.as_string()
				.unwrap_or(StringRef::from("-"))
				.read(),
			description: String::from(""),
			url: String::from(""),
			categories: Vec::new(),
			status: match manga_obj.get("mangaIsOver").as_int().unwrap_or(-1) {
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

// {
//   sectionId: 738001,
//   sectionName: '第89卷',
//   sectionIsNewest: 0,
//   sectionOfflineUrl:
// 'mangaapi.manhuaren.com/comicChapterDownLoad.ashx?cid=738001',   sectionType:
// 0,   sectionUrl: '',
//   sectionTitle: '',
//   sectionSort: 89,
//   sectionSubName: '89',
//   isMustPay: 0,
//   authority: 0,
//   hasUnlockDate: 0,
//   releaseTime: '2018-12-12',
//   beFreeSince: '',
//   imageUrl: '',
//   isNoAllowDownload: 0,
//   otherSectionId: '',
//   mangaLanguage: 0,
//   sectionSign: 0,
//   uploadUserId: 0
// }
fn parse_chapters(manga: &ObjectRef, key: &str) -> Vec<Chapter> {
	match manga.get(key).as_array() {
		Ok(chapters) => {
			let mut chapter_arr: Vec<Chapter> = Vec::new();

			for ch in chapters {
				let ch_obj = ch.as_object().unwrap();
				let mut title = String::new();

				if ch_obj.get("isMustPay").as_int().unwrap_or(0) == 1 {
					title.push_str("(锁) ");
				}

				if key.eq("mangaEpisode") {
					title.push_str("[番外] ");
				}

				let section_name = ch_obj
					.get("sectionName")
					.as_string()
					.unwrap_or(StringRef::from(""))
					.read();
				let section_title = ch_obj
					.get("sectionTitle")
					.as_string()
					.unwrap_or(StringRef::from(""))
					.read();

				title.push_str(&section_name);

				if !section_title.is_empty() {
					title.push_str(": ");
					title.push_str(&section_title);
				}

				chapter_arr.push(Chapter {
					id: helper::i32_to_string(ch_obj.get("sectionId").as_int().unwrap_or(0) as i32),
					title,
					volume: -1.0,
					chapter: ch_obj.get("sectionSort").as_float().unwrap_or(0.0) as f32,
					date_updated: match ch_obj.get("releaseTime").as_date(
						"yyyy-MM-dd",
						Option::from("zh"),
						Option::from("TW"),
					) {
						Ok(d) => d,
						_ => -1.0,
					},
					scanlator: String::from(""),
					url: String::from(""),
					lang: String::from("zh"),
				})
			}

			return chapter_arr;
		}
		_ => Vec::new(),
	}
}

fn parse_page(chapter: &ObjectRef) -> Vec<Page> {
	match chapter.get("mangaSectionImages").as_array() {
		Ok(pages) => {
			let host_list = chapter.get("hostList").as_array().unwrap();
			let host = host_list
				.get(0)
				.as_string()
				.unwrap_or(StringRef::from(""))
				.read();
			let query = chapter
				.get("query")
				.as_string()
				.unwrap_or(StringRef::from(""))
				.read();

			let mut page_arr: Vec<Page> = Vec::new();

			for (i, p) in pages.enumerate() {
				let p_str = p.as_string().unwrap_or(StringRef::from("")).read();

				let mut url = String::from(helper::encode_uri(String::from(&host)));
				url.push_str(&p_str);
				url.push_str(&query);

				// print(&url);

				page_arr.push(Page {
					index: (i + 1) as i32,
					url,
					base64: String::from(""),
					text: String::from(""),
				});
			}
			return page_arr;
		}
		_ => Vec::new(),
	}
}

struct ListFilter {
	status: i32,
	genre: i32,
	sort: i32,
}
