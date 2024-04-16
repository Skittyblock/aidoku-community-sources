extern crate alloc;

use aidoku::{
	helpers::uri::QueryParameters,
	std::{defaults::defaults_get, net::Request, String, Vec},
	Filter, FilterType,
};
use alloc::string::ToString;
use core::fmt::Display;

pub enum Url<'a> {
	/// https://{subdomain}.baozimh.com
	///
	/// ---
	///
	/// ## subdomain
	///
	/// - `cn`: 简体
	/// - `www`: 預設
	/// - `tw`: 繁體
	Domain,

	/// {Url::Domain}/api/bzmhq/amp_comic_list?{query}
	///
	/// ---
	///
	/// ## query
	///
	/// ### `type`
	///
	/// - `all`: 全部
	/// - `lianai`: 戀愛
	/// - `chunai`: 純愛
	/// - `gufeng`: 古風
	/// - `yineng`: 異能
	/// - `xuanyi`: 懸疑
	/// - `juqing`: 劇情
	/// - `kehuan`: 科幻
	/// - `qihuan`: 奇幻
	/// - `xuanhuan`: 玄幻
	/// - `chuanyue`: 穿越
	/// - `mouxian`: 冒險
	/// - `tuili`: 推理
	/// - `wuxia`: 武俠
	/// - `gedou`: 格鬥
	/// - `zhanzheng`: 戰爭
	/// - `rexie`: 熱血
	/// - `gaoxiao`: 搞笑
	/// - `danuzhu`: 大女主
	/// - `dushi`: 都市
	/// - `zongcai`: 總裁
	/// - `hougong`: 後宮
	/// - `richang`: 日常
	/// - `hanman`: 韓漫
	/// - `shaonian`: 少年
	/// - `qita`: 其他
	///
	/// ### `region`
	///
	/// - `all`: 全部
	/// - `cn`: 中港台
	/// - `jp`: 日本
	/// - `kr`: 韓國
	/// - `en`: 歐美
	///
	/// ### `filter`
	///
	/// - `*`: 全部
	/// - `ABCD`
	/// - `EFGH`
	/// - `IJKL`
	/// - `MNOP`
	/// - `QRST`
	/// - `UVW`
	/// - `XYZ`
	/// - `0-9`
	///
	/// ### `page`
	///
	/// Start from `1`
	Filters(QueryParameters),

	/// {Url::Domain}/search?{query}
	///
	/// ---
	///
	/// ## query
	///
	/// ### `q`
	///
	/// `search_str` ➡️ Should be percent-encoded
	Search(QueryParameters),

	/// https://static-tw.baozimh.com/cover/{topic_img}
	Cover(&'a str),

	/// {Url::Domain}/comic/{manga_id}
	Manga(&'a str),

	/// {Url::Domain}/user/page_direct?{query}
	///
	/// ## query
	///
	/// ### `comic_id`
	///
	/// `manga_id`
	///
	/// ### `section_slot`
	///
	/// `0`
	///
	/// ### `chapter_slot`
	///
	/// `chapter_id`
	Chapter(QueryParameters),

	/// {Url::Domain}/list/new
	New,
}

impl Url<'_> {
	pub fn get(self) -> Request {
		Request::get(self.to_string())
	}
}

impl Display for Url<'_> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::Domain => {
				let charset_index = defaults_get("charset")
					.and_then(|value| value.as_int())
					.unwrap_or(1);
				let charset = match charset_index {
					// ! Cloudflare issue
					// 0 => "cn",
					1 => "www",
					2 => "tw",
					_ => "www",
				};

				write!(f, "https://{}.baozimh.com", charset)
			}

			Self::Filters(query) => {
				write!(f, "{}/api/bzmhq/amp_comic_list?{}", Self::Domain, query)
			}

			Self::Search(query) => write!(f, "{}/search?{}", Self::Domain, query),

			Self::Cover(topic_img) => {
				write!(f, "https://static-tw.baozimh.com/cover/{}", topic_img)
			}

			Self::Manga(manga_id) => write!(f, "{}/comic/{}", Self::Domain, manga_id),

			Self::Chapter(query) => write!(f, "{}/user/page_direct?{}", Self::Domain, query),

			Self::New => write!(f, "{}/list/new", Self::Domain),
		}
	}
}

impl From<(Vec<Filter>, i32)> for Url<'_> {
	fn from((filters, page): (Vec<Filter>, i32)) -> Self {
		let mut filters_query = QueryParameters::new();

		for filter in filters {
			match filter.kind {
				FilterType::Select => {
					let index = filter.value.as_int().unwrap_or(0) as usize;

					match filter.name.as_str() {
						"類型" => {
							let genres = [
								"all",
								"lianai",
								"chunai",
								"gufeng",
								"yineng",
								"xuanyi",
								"juqing",
								"kehuan",
								"qihuan",
								"xuanhuan",
								"chuanyue",
								"mouxian",
								"tuili",
								"wuxia",
								"gedou",
								"zhanzheng",
								"rexie",
								"gaoxiao",
								"danuzhu",
								"dushi",
								"zongcai",
								"hougong",
								"richang",
								"hanman",
								"shaonian",
								"qita",
							];

							filters_query.push_encoded("type", Some(genres[index]));
						}

						"地區" => {
							let regions = ["all", "cn", "jp", "kr", "en"];

							filters_query.push_encoded("region", Some(regions[index]));
						}

						"字母" => {
							let letters = [
								"*", "ABCD", "EFGH", "IJKL", "MNOP", "QRST", "UVW", "XYZ", "0-9",
							];

							filters_query.push_encoded("filter", Some(letters[index]));
						}

						_ => continue,
					}
				}

				FilterType::Title => {
					let Ok(search_str) = filter.value.as_string().map(|str_ref| str_ref.read())
					else {
						continue;
					};

					let mut search_query = QueryParameters::new();
					search_query.push("q", Some(&search_str));

					return Url::Search(search_query);
				}

				_ => continue,
			}
		}

		filters_query.push_encoded("page", Some(page.to_string().as_str()));

		Url::Filters(filters_query)
	}
}

impl From<Url<'_>> for String {
	fn from(url: Url<'_>) -> Self {
		url.to_string()
	}
}
