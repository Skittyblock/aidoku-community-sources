#![no_std]
use aidoku::{
	error::Result,
	helpers::{substring::Substring, uri::encode_uri_component},
	prelude::*,
	std::{html::unescape_html_entities, net::Request, String, Vec},
	Chapter, DeepLink, Filter,
	FilterType::{Genre, Select, Sort, Title},
	Manga, MangaContentRating, MangaPageResult,
	MangaStatus::{Completed, Ongoing, Unknown},
	Page,
};

extern crate alloc;
use alloc::string::ToString;

enum Url<'a> {
	/// https://boylove.cc{path}
	Abs(&'a str),

	/// https://boylove.cc/home/api/searchk?keyword={}&type={}&pageNo={}
	///
	/// ---
	///
	/// `keyword` ➡️ Should be percent-encoded
	///
	/// `type`:
	///
	/// - **`1`: 漫畫** ➡️ Always
	/// - `2`: 小說
	///
	/// `pageNo`: Start from `1`
	Search(&'a str, i32),

	/// https://boylove.cc/home/api/cate/tp/1-{tags}-{status}-{sort_by}-{page}-{content_rating}-{content_type}-{viewing_permission}
	///
	/// ---
	///
	/// `content_type`:
	///
	/// - **`1`: 漫畫** ➡️ Always
	/// - `2`: 小說
	///
	/// `viewing_permission`:
	///
	/// - `2`: 全部
	/// - **`0`: 一般** ➡️ Always
	/// - ~~`1`: VIP~~ ➡️ Login cookie is required to view manga for VIP members
	Filters {
		/// - `0`: 全部
		/// - `A+B+…+Z` ➡️ Should be percent-encoded
		tags: &'a str,

		/// - `2`: 全部
		/// - `0`: 連載中
		/// - `1`: 已完結
		status: u8,

		/// - `0`: 人氣 ➡️ ❗️**Not sure**❗️
		/// - `1`: 最新更新
		sort_by: u8,

		/// Start from `1`
		page: i32,

		/// - `0`: 全部
		/// - `1`: 清水
		/// - `2`: 有肉
		content_rating: u8,
		// //
		// // viewing_permission: u8,
	},

	/// https://boylove.cc/home/api/chapter_list/tp/{manga_id}-0-0-10
	ChapterList(&'a str),

	/// https://boylove.cc/home/book/index/id/{manga_id}
	Manga(&'a str),

	/// https://boylove.cc/home/book/capter/id/{chapter_id}
	Chapter(&'a str),
}

const DOMAIN: &str = "https://boylove.cc";
const MANGA_PATH: &str = "index/id/";
const CHAPTER_PATH: &str = "capter/id/";

/// Chrome 114 on macOS
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36";

/// 連載狀態：\[全部, 連載中, 已完結\]
const FILTER_STATUS: [u8; 3] = [2, 0, 1];

/// 內容分級：\[全部, 清水, 有肉\]
const FILTER_CONTENT_RATING: [u8; 3] = [0, 1, 2];

/// 排序依據：\[最新更新, 人氣\]
const SORT: [u8; 2] = [1, 0];

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let manga_list_url = get_filtered_url(filters, page)?;
	let manga_list_json = request_get(&manga_list_url).json()?;
	let manga_list_obj = manga_list_json.as_object()?;
	let result = manga_list_obj.get("result").as_object()?;

	let mut manga = Vec::<Manga>::new();
	let manga_arr = result.get("list").as_array()?;
	for manga_value in manga_arr {
		let manga_obj = manga_value.as_object()?;
		let keyword = manga_obj.get("keyword").as_string()?.read();

		// !! There's an ad whose lanmu_id is not 5, DO NOT use
		// // let is_ad = manga_obj.get("lanmu_id").as_int().unwrap_or(0) == 5;
		let is_ad = keyword.contains("公告");
		if is_ad {
			continue;
		}

		let manga_id = manga_obj.get("id").as_int()?.to_string();

		let cover_path = manga_obj.get("image").as_string()?.read();
		let cover_url = Url::Abs(&cover_path).to_string();

		let manga_title = manga_obj.get("title").as_string()?.read();

		let artists_str = manga_obj
			.get("auther")
			.as_string()?
			.read()
			.replace('&', "、");

		let description = manga_obj.get("desc").as_string()?.read();

		let manga_url = Url::Manga(&manga_id).to_string();

		let categories = keyword
			.split(',')
			.filter(|tag| !tag.is_empty())
			.map(ToString::to_string)
			.collect::<Vec<String>>();

		let status = match manga_obj.get("mhstatus").as_int()? {
			0 => Ongoing,
			1 => Completed,
			_ => Unknown,
		};

		let content_rating = get_content_rating(&categories);

		manga.push(Manga {
			id: manga_id,
			cover: cover_url,
			title: manga_title,
			author: artists_str.clone(),
			artist: artists_str,
			description,
			url: manga_url,
			categories,
			status,
			nsfw: content_rating,
			..Default::default()
		});
	}

	let has_more = !result.get("lastPage").as_bool()?;

	Ok(MangaPageResult { manga, has_more })
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let manga_url = Url::Manga(&manga_id).to_string();

	let manga_html = request_get(&manga_url).html()?;

	let cover_url = manga_html.select("a.play").attr("abs:data-original").read();

	let manga_title = manga_html.select("div.title > h1").text().read();

	let artists_str = manga_html
		.select("p.data:contains(作者：) > a")
		.array()
		.filter_map(Parser::get_is_ok_text)
		.collect::<Vec<String>>()
		.join("、");

	let mut description =
		unescape_html_entities(manga_html.select("span.detail-text").html().read())
			.split("<br>")
			.map(str::trim)
			.collect::<Vec<&str>>()
			.join("\n")
			.trim()
			.to_string();
	if let Some(description_removed_closing_tag) = description.substring_before_last("</") {
		description = description_removed_closing_tag.trim().to_string();
	}

	let categories = manga_html
		.select("a.tag > span")
		.array()
		.filter_map(Parser::get_is_ok_text)
		.filter(|tag| !tag.is_empty())
		.collect::<Vec<String>>();

	let status = match manga_html.select("p.data").first().text().read().as_str() {
		"连载中" => Ongoing,
		"完结" => Completed,
		_ => Unknown,
	};

	let content_rating = get_content_rating(&categories);

	Ok(Manga {
		id: manga_id,
		cover: cover_url,
		title: manga_title,
		author: artists_str.clone(),
		artist: artists_str,
		description,
		url: manga_url,
		categories,
		status,
		nsfw: content_rating,
		..Default::default()
	})
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let chapter_list_url = Url::ChapterList(&manga_id).to_string();
	let chapter_list_json = request_get(&chapter_list_url).json()?;
	let chapter_list_obj = chapter_list_json.as_object()?;
	let result = chapter_list_obj.get("result").as_object()?;

	let mut chapters = Vec::<Chapter>::new();
	let chapters_arr = result.get("list").as_array()?;
	for (chapter_index, chapter_value) in chapters_arr.rev().enumerate() {
		let chapter_obj = chapter_value.as_object()?;

		let chapter_id = chapter_obj.get("id").as_int()?.to_string();

		let chapter_title = chapter_obj.get("title").as_string()?.read();

		let chapter_num = (chapter_index + 1) as f32;

		let chapter_url = Url::Chapter(&chapter_id).to_string();

		let chapter = Chapter {
			id: chapter_id,
			title: chapter_title,
			chapter: chapter_num,
			url: chapter_url,
			lang: "zh".to_string(),
			..Default::default()
		};
		chapters.insert(0, chapter);
	}

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let chapter_url = Url::Chapter(&chapter_id).to_string();
	let chapter_html = request_get(&chapter_url).html()?;

	let mut pages = Vec::<Page>::new();
	let page_nodes = chapter_html.select("img.lazy[id]");
	for (page_index, page_value) in page_nodes.array().enumerate() {
		let page_path = page_value
			.as_node()?
			.attr("data-original")
			.read()
			.trim()
			.to_string();
		let page_url = Url::Abs(&page_path).to_string();

		pages.push(Page {
			index: page_index as i32,
			url: page_url,
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

	let chapter_html = request_get(&url).html()?;
	let manga_url = chapter_html
		.select("a.icon-only.link.back")
		.attr("href")
		.read();
	let Some(manga_id) = manga_url.substring_after_last("/") else {
		return Ok(DeepLink {
			manga: None,
			chapter,
		});
	};
	let manga = Some(get_manga_details(manga_id.to_string())?);

	Ok(DeepLink { manga, chapter })
}

fn get_filtered_url(filters: Vec<Filter>, page: i32) -> Result<String> {
	let mut filter_status = FILTER_STATUS[0];
	let mut filter_content_rating = FILTER_CONTENT_RATING[0];
	let mut filter_tags_vec = Vec::<String>::new();
	let mut sort_by = SORT[0];

	for filter in filters {
		match filter.kind {
			Select => {
				let index = filter.value.as_int().unwrap_or(0) as usize;
				match filter.name.as_str() {
					"連載狀態" => filter_status = FILTER_STATUS[index],
					"內容分級" => filter_content_rating = FILTER_CONTENT_RATING[index],
					_ => continue,
				}
			}

			Sort => {
				let obj = filter.value.as_object()?;
				let index = obj.get("index").as_int().unwrap_or(0) as usize;
				sort_by = SORT[index];
			}

			Title => {
				let encoded_search_str = encode_uri_component(filter.value.as_string()?.read());

				return Ok(Url::Search(&encoded_search_str, page).to_string());
			}

			Genre => {
				let is_not_checked = filter.value.as_int().unwrap_or(-1) != 1;
				if is_not_checked {
					continue;
				}

				let encoded_tag = encode_uri_component(filter.name);
				filter_tags_vec.push(encoded_tag);
			}

			_ => continue,
		}
	}

	let filter_tags_str = match filter_tags_vec.is_empty() {
		// ? 全部
		true => "0".to_string(),

		false => filter_tags_vec.join("+"),
	};

	Ok(Url::Filters {
		tags: &filter_tags_str,
		status: filter_status,
		sort_by,
		page,
		content_rating: filter_content_rating,
	}
	.to_string())
}

/// Start a new GET request with the given URL with headers `Referer` and
/// `User-Agent` set.
fn request_get(url: &str) -> Request {
	Request::get(url)
		.header("Referer", DOMAIN)
		.header("User-Agent", USER_AGENT)
}

/// Returns [`Safe`](MangaContentRating::Safe) if the given slice contains
/// `清水`, or else returns [`Nsfw`](MangaContentRating::Nsfw).
fn get_content_rating(categories: &[String]) -> MangaContentRating {
	if categories.contains(&"清水".to_string()) {
		return MangaContentRating::Safe;
	}
	MangaContentRating::Nsfw
}

impl core::fmt::Display for Url<'_> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		let api_path = format!("{}/home/api/", DOMAIN);
		let html_path = format!("{}/home/book/", DOMAIN);

		match self {
			Self::Abs(path) => write!(f, "{}{}", DOMAIN, path),

			Self::Search(search_str, page) => write!(
				f,
				"{}searchk?keyword={}&type=1&pageNo={}",
				api_path, search_str, page
			),

			Self::Filters {
				tags,
				status,
				sort_by,
				page,
				content_rating,
			} => write!(
				f,
				"{}cate/tp/1-{}-{}-{}-{}-{}-1-0",
				api_path, tags, status, sort_by, page, content_rating,
			),

			Self::ChapterList(manga_id) => {
				write!(f, "{}chapter_list/tp/{}-0-0-10", api_path, manga_id)
			}

			Self::Manga(manga_id) => write!(f, "{}{}{}", html_path, MANGA_PATH, manga_id),

			Self::Chapter(chapter_id) => write!(f, "{}{}{}", html_path, CHAPTER_PATH, chapter_id),
		}
	}
}

trait Parser {
	/// Returns [`None`], or the text of the Node (if [`Ok`]).
	fn get_is_ok_text(self) -> Option<String>;
}

impl Parser for aidoku::std::ValueRef {
	fn get_is_ok_text(self) -> Option<String> {
		self.as_node().map_or(None, |node| Some(node.text().read()))
	}
}
