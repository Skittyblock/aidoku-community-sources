use aidoku::{
	error::Result,
	helpers::uri::QueryParameters,
	prelude::format,
	std::{html::Node, net::Request, String, ValueRef, Vec},
	Filter, FilterType,
};
use alloc::{borrow::ToOwned as _, string::ToString};
use core::fmt::{Display, Formatter, Result as FmtResult};
use strum_macros::{Display, IntoStaticStr};

#[expect(private_interfaces)]
#[derive(Display)]
#[strum(prefix = "https://mangacopy.com")]
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

	#[strum(to_string = "/search")]
	SearchPage,

	#[strum(to_string = "{search}")]
	Search { search: Search },

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

#[expect(dead_code)]
#[derive(Default, IntoStaticStr, Clone, Copy)]
enum SearchType {
	#[default]
	#[strum(to_string = "")]
	All,

	#[strum(to_string = "name")]
	Title,

	#[strum(to_string = "author")]
	Author,

	#[strum(to_string = "local")]
	Translator,
}

#[derive(Default)]
struct Search {
	page: i32,
	keyword: String,
	by: SearchType,
}

impl Search {
	fn new<S: AsRef<str>>(page: i32, keyword: S) -> Self {
		Self {
			page,
			keyword: keyword.as_ref().to_owned(),
			..Default::default()
		}
	}
}

impl Display for Search {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let path = Url::SearchPage
			.get_html()
			.ok()
			.and_then(|page| {
				let count_api = page
					.html()
					.read()
					.lines()
					.find(|line| line.contains("const countApi"))?
					.split('"')
					.nth(1)?
					.to_owned();

				Some(count_api)
			})
			.unwrap_or_else(|| "/api/kb/web/searchbc/comics".into());

		let mut query = QueryParameters::new();

		let offset = self.page.checked_sub(1).unwrap_or(0).saturating_mul(LIMIT);
		query.push_encoded("offset", Some(&offset.to_string()));

		query.push_encoded("platform", Some(&2.to_string()));

		query.push_encoded("limit", Some(&LIMIT.to_string()));

		query.push("q", Some(&self.keyword));

		let search_by = (!matches!(self.by, SearchType::All)).then(|| self.by.into());
		query.push_encoded("q_type", search_by);

		write!(f, "{path}?{query}")
	}
}

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

impl Url<'_> {
	pub fn get_html(self) -> Result<Node> {
		self.get().html()
	}

	pub fn get_json(self) -> Result<ValueRef> {
		self.get().json()
	}

	fn get(self) -> Request {
		Request::get(self.to_string()).header(
			"User-Agent",
			"Mozilla/5.0 (Macintosh; Intel Mac OS X 14_7_4) \
			 AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.3 Safari/605.1.15",
		)
	}
}

impl From<(Vec<Filter>, i32)> for Url<'_> {
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
					let keyword = match filter.value.as_string() {
						Ok(str_ref) => str_ref.read(),
						Err(_) => continue,
					};
					let search = Search::new(page, keyword);

					return Url::Search { search };
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
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::DateUpdated(is_asc) => {
				write!(f, "{}datetime_updated", if *is_asc { "" } else { "-" })
			}
			Self::Popularity(is_asc) => write!(f, "{}popular", if *is_asc { "" } else { "-" }),
		}
	}
}
