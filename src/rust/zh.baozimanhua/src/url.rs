extern crate alloc;

use aidoku::{helpers::uri::QueryParameters, std::Vec, Filter, FilterType};
use alloc::string::ToString;
use core::fmt::Display;

const DOMAIN: &str = "https://www.baozimh.com";

pub enum Url {
	/// {DOMAIN}/api/bzmhq/amp_comic_list?{query}
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

	/// {DOMAIN}/search?{query}
	///
	/// ---
	///
	/// ## query
	///
	/// ### `q`
	///
	/// Should be percent-encoded
	Search(QueryParameters),
}

impl From<(Vec<Filter>, i32)> for Url {
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

impl Display for Url {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::Filters(query) => write!(f, "{}/api/bzmhq/amp_comic_list?{}", DOMAIN, query),
			Self::Search(query) => write!(f, "{}/search?{}", DOMAIN, query),
		}
	}
}
