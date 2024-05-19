use aidoku::{
	error::Result,
	helpers::uri::QueryParameters,
	prelude::format,
	std::{html::Node, net::Request, ValueRef, Vec},
	Filter, FilterType,
};
use alloc::string::ToString;
use core::fmt::Display;
use strum_macros::Display;

#[derive(Display)]
#[strum(prefix = "https://copymanga.site")]
pub enum Url<'a> {
	/// ## `theme`
	///
	/// - : 全部
	/// - `aiqing`: 愛情
	/// - `huanlexiang`: 歡樂向
	/// - `maoxian`: 冒險
	/// - `qihuan`: 奇幻
	/// - `baihe`: 百合
	/// - `xiaoyuan`: 校園
	/// - `kehuan`: 科幻
	/// - `dongfang`: 東方
	/// - `danmei`: 耽美
	/// - `shenghuo`: 生活
	/// - `gedou`: 格鬥
	/// - `qingxiaoshuo`: 輕小說
	/// - `xuanyi`: 懸疑
	/// - `qita`: 其他
	/// - `shengui`: 神鬼
	/// - `zhichang`: 職場
	/// - `teenslove`: TL
	/// - `mengxi`: 萌系
	/// - `zhiyu`: 治癒
	/// - `changtiao`: 長條
	/// - `sige`: 四格
	/// - `jiecao`: 節操
	/// - `jianniang`: 艦娘
	/// - `jingji`: 競技
	/// - `gaoxiao`: 搞笑
	/// - `weiniang`: 偽娘
	/// - `rexue`: 熱血
	/// - `lizhi`: 勵志
	/// - `xingzhuanhuan`: 性轉換
	/// - `COLOR`: 彩色
	/// - `hougong`: 後宮
	/// - `meishi`: 美食
	/// - `zhentan`: 偵探
	/// - `aa`: AA
	/// - `yinyuewudao`: 音樂舞蹈
	/// - `mohuan`: 魔幻
	/// - `zhanzheng`: 戰爭
	/// - `lishi`: 歷史
	/// - `yishijie`: 異世界
	/// - `jingsong`: 驚悚
	/// - `jizhan`: 機戰
	/// - `dushi`: 都市
	/// - `chuanyue`: 穿越
	/// - `kongbu`: 恐怖
	/// - `comiket100`: C100
	/// - `chongsheng`: 重生
	/// - `comiket99`: C99
	/// - `comiket101`: C101
	/// - `comiket97`: C97
	/// - `comiket96`: C96
	/// - `shengcun`: 生存
	/// - `zhaixi`: 宅系
	/// - `wuxia`: 武俠
	/// - `C98`: C98
	/// - `comiket95`: C95
	/// - `fate`: FATE
	/// - `zhuansheng`: 轉生
	/// - `Uncensored`: 無修正
	/// - `xianxia`: 仙俠
	/// - `loveLive`: LoveLive
	///
	/// ## `status`
	///
	/// - : 全部
	/// - `0`: 連載中
	/// - `1`: 已完結
	/// - `2`: 短篇
	///
	/// ## `region`
	///
	/// - : 全部
	/// - `0`: 日漫
	/// - `1`: 韓漫
	/// - `2`: 美漫
	///
	/// ## `ordering`
	///
	/// `{order}{sort_by}`
	///
	/// ### `order`
	///
	/// - `-`: 降冪
	/// - : 升冪
	///
	/// ### `sort_by`
	///
	/// - `datetime_updated`: 更新時間
	/// - `popular`: 熱門
	///
	/// ## `offset`
	///
	/// `({page} - 1) * {limit}`
	///
	/// ## `limit`
	///
	/// Manga per response
	#[strum(to_string = "/comics?{query}")]
	Filters { query: QueryParameters },

	/// ## `offset`
	///
	/// `({page} - 1) * {limit}`
	///
	/// ## `platform`
	///
	/// `2`
	///
	/// ## `limit`
	///
	/// Manga per response
	///
	/// ## `q`
	///
	/// `search_str` ➡️ Should be percent-encoded
	///
	/// ## `q_type`
	///
	/// - ``: 全部
	/// - `name`: 名稱
	/// - `author`: 作者
	/// - `local`: 漢化組
	#[strum(to_string = "/api/kb/web/searchb/comics?{query}")]
	Search { query: QueryParameters },

	#[strum(to_string = "/comic/{id}")]
	Manga { id: &'a str },

	#[strum(to_string = "/comicdetail/{id}/chapters")]
	ChapterList { id: &'a str },

	#[strum(to_string = "/comic/{manga_id}/chapter/{chapter_id}")]
	Chapter {
		manga_id: &'a str,
		chapter_id: &'a str,
	},
}

/// # 狀態
#[derive(Copy, Clone)]
enum Status {
	/// ## 全部
	All = -1,

	/// ## 連載中
	Ongoing = 0,

	/// ## 已完結
	Completed = 1,

	/// ## 短篇
	OneShot = 2,
}

/// # 地區
#[derive(Copy, Clone)]
enum Region {
	/// ## 全部
	All = -1,

	/// ## 日漫
	Japan = 0,

	/// ## 韓漫
	Korea = 1,

	/// ## 美漫
	West = 2,
}

/// # 排序
enum Sort {
	/// ## 更新時間
	///
	/// - `true`: 升冪
	/// - `false`: 降冪
	DateUpdated(bool),

	/// ## 熱門
	///
	/// - `true`: 升冪
	/// - `false`: 降冪
	Popularity(bool),
}

pub const MANGA_PATH: &str = "/comic/";
pub const CHAPTER_PATH: &str = "/chapter/";

const GENRES: [&str; 61] = [
	"",
	"aiqing",
	"huanlexiang",
	"maoxian",
	"qihuan",
	"baihe",
	"xiaoyuan",
	"kehuan",
	"dongfang",
	"danmei",
	"shenghuo",
	"gedou",
	"qingxiaoshuo",
	"xuanyi",
	"qita",
	"shengui",
	"zhichang",
	"teenslove",
	"mengxi",
	"zhiyu",
	"changtiao",
	"sige",
	"jiecao",
	"jianniang",
	"jingji",
	"gaoxiao",
	"weiniang",
	"rexue",
	"lizhi",
	"xingzhuanhuan",
	"COLOR",
	"hougong",
	"meishi",
	"zhentan",
	"aa",
	"yinyuewudao",
	"mohuan",
	"zhanzheng",
	"lishi",
	"yishijie",
	"jingsong",
	"jizhan",
	"dushi",
	"chuanyue",
	"kongbu",
	"comiket100",
	"chongsheng",
	"comiket99",
	"comiket101",
	"comiket97",
	"comiket96",
	"shengcun",
	"zhaixi",
	"wuxia",
	"C98",
	"comiket95",
	"fate",
	"zhuansheng",
	"Uncensored",
	"xianxia",
	"loveLive",
];
const STATUSES: [Status; 4] = [
	Status::All,
	Status::Ongoing,
	Status::Completed,
	Status::OneShot,
];
const REGIONS: [Region; 4] = [Region::All, Region::Japan, Region::Korea, Region::West];

/// The number of manga that a single response contains.
const LIMIT: i32 = 20;

impl<'a> Url<'a> {
	pub fn get_html(self) -> Result<Node> {
		Request::get(self.to_string()).html()
	}

	pub fn get_json(self) -> Result<ValueRef> {
		Request::get(self.to_string()).json()
	}
}

impl<'a> From<(Vec<Filter>, i32)> for Url<'a> {
	fn from((filters, page): (Vec<Filter>, i32)) -> Self {
		let mut genre_index = 0;
		let mut region = Region::All;
		let mut status = Status::All;
		let mut sort_by = Sort::DateUpdated(false);

		let mut query = QueryParameters::new();

		let offset = (page - 1) * LIMIT;
		query.push_encoded("offset", Some(offset.to_string().as_str()));
		query.push_encoded("limit", Some(LIMIT.to_string().as_str()));

		for filter in filters {
			match filter.kind {
				FilterType::Select => {
					let index = filter.value.as_int().unwrap_or(0) as usize;
					match filter.name.as_str() {
						"題材" => genre_index = index as u8,
						"地區" => region = REGIONS[index],
						"狀態" => status = STATUSES[index],
						_ => continue,
					}
				}

				FilterType::Sort => {
					let obj_result = filter.value.as_object();

					let index = obj_result
						.clone()
						.and_then(|obj| obj.get("index").as_int())
						.unwrap_or(0);

					let is_asc = obj_result
						.and_then(|obj| obj.get("ascending").as_bool())
						.unwrap_or(false);

					sort_by = match index {
						1 => Sort::Popularity(is_asc),
						_ => Sort::DateUpdated(is_asc),
					};
				}

				FilterType::Title => {
					let Ok(search_str_ref) = filter.value.as_string() else {
						continue;
					};
					let search_str = search_str_ref.read();

					query.push_encoded("platform", Some(2.to_string().as_str()));
					query.push("q", Some(&search_str));
					query.push_encoded("q_type", None);

					return Url::Search { query };
				}

				_ => continue,
			}
		}

		query.push_encoded("theme", Some(GENRES[genre_index as usize]));
		query.push_encoded("status", Some(status.to_string().as_str()));
		query.push_encoded("region", Some(region.to_string().as_str()));
		query.push_encoded("ordering", Some(sort_by.to_string().as_str()));

		Url::Filters { query }
	}
}

/// Implement `Display` for filter enums.
macro_rules! impl_display {
	($filter: ident) => {
		impl Display for $filter {
			fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
				match self {
					Self::All => write!(f, ""),
					_ => write!(f, "{}", *self as u8),
				}
			}
		}
	};
}
impl_display!(Status);
impl_display!(Region);

impl Display for Sort {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::DateUpdated(is_asc) => {
				write!(f, "{}datetime_updated", if *is_asc { "" } else { "-" })
			}
			Self::Popularity(is_asc) => write!(f, "{}popular", if *is_asc { "" } else { "-" }),
		}
	}
}
