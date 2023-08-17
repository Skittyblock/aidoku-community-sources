use aidoku::{
	error::Result,
	helpers::uri::encode_uri_component,
	prelude::format,
	std::{String, Vec},
	Filter, FilterType,
};
use alloc::string::ToString;
use core::fmt::Display;

pub const DOMAIN: &str = "https://boylove.cc";
pub const MANGA_PATH: &str = "index/id/";
pub const CHAPTER_PATH: &str = "capter/id/";

/// 閱覽權限：\[全部, 一般, VIP\]
const FILTER_VIEWING_PERMISSION: [u8; 3] = [2, 0, 1];

/// 連載狀態：\[全部, 連載中, 已完結\]
const FILTER_STATUS: [u8; 3] = [2, 0, 1];

/// 內容分級：\[全部, 清水, 有肉\]
const FILTER_CONTENT_RATING: [u8; 3] = [0, 1, 2];

/// 排序依據：\[最新更新, 人氣\]
const SORT: [u8; 2] = [1, 0];

pub enum Url<'a> {
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
	Search(String, i32),

	/// https://boylove.cc/home/api/cate/tp/1-{tags}-{status}-{sort_by}-{page}-{content_rating}-{content_type}-{viewing_permission}
	///
	/// ---
	///
	/// `content_type`:
	///
	/// - **`1`: 漫畫** ➡️ Always
	/// - `2`: 小說
	Filters {
		/// - `0`: 全部
		/// - `A+B+…+Z` ➡️ Should be percent-encoded
		tags: String,

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

		/// - `2`: 全部
		/// - `0`: 一般
		/// - `1`: VIP
		viewing_permission: u8,
	},

	/// https://boylove.cc/home/api/chapter_list/tp/{manga_id}-0-0-10
	ChapterList(String),

	/// https://boylove.cc/home/book/index/id/{manga_id}
	Manga(&'a str),

	/// https://boylove.cc/home/book/capter/id/{chapter_id}
	Chapter(&'a str),
}

impl<'a> Url<'a> {
	pub fn from(filters: Vec<Filter>, page: i32) -> Result<Self> {
		let mut filter_viewing_permission = FILTER_VIEWING_PERMISSION[0];
		let mut filter_status = FILTER_STATUS[0];
		let mut filter_content_rating = FILTER_CONTENT_RATING[0];
		let mut filter_tags_vec = Vec::<String>::new();
		let mut sort_by = SORT[0];

		for filter in filters {
			match filter.kind {
				FilterType::Select => {
					let index = filter.value.as_int().unwrap_or(0) as usize;
					match filter.name.as_str() {
						"閱覽權限" => {
							filter_viewing_permission = FILTER_VIEWING_PERMISSION[index];
						}
						"連載狀態" => filter_status = FILTER_STATUS[index],
						"內容分級" => filter_content_rating = FILTER_CONTENT_RATING[index],
						_ => continue,
					}
				}

				FilterType::Sort => {
					let obj = filter.value.as_object()?;
					let index = obj.get("index").as_int().unwrap_or(0) as usize;
					sort_by = SORT[index];
				}

				FilterType::Title => {
					let encoded_search_str = encode_uri_component(filter.value.as_string()?.read());

					return Ok(Url::Search(encoded_search_str, page));
				}

				FilterType::Genre => {
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

		let filter_tags_str = if filter_tags_vec.is_empty() {
			// ? 全部
			"0".to_string()
		} else {
			filter_tags_vec.join("+")
		};

		Ok(Url::Filters {
			tags: filter_tags_str,
			status: filter_status,
			sort_by,
			page,
			content_rating: filter_content_rating,
			viewing_permission: filter_viewing_permission,
		})
	}
}

impl<'a> Display for Url<'a> {
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
				viewing_permission,
			} => write!(
				f,
				"{}cate/tp/1-{}-{}-{}-{}-{}-1-{}",
				api_path, tags, status, sort_by, page, content_rating, viewing_permission
			),

			Self::ChapterList(manga_id) => {
				write!(f, "{}chapter_list/tp/{}-0-0-10", api_path, manga_id)
			}

			Self::Manga(manga_id) => write!(f, "{}{}{}", html_path, MANGA_PATH, manga_id),

			Self::Chapter(chapter_id) => write!(f, "{}{}{}", html_path, CHAPTER_PATH, chapter_id),
		}
	}
}
