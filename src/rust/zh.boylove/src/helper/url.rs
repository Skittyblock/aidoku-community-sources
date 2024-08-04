use aidoku::{
	helpers::uri::{encode_uri_component, QueryParameters},
	prelude::format,
	std::{net::Request, String, Vec},
	Filter, FilterType,
};
use alloc::string::ToString;
use core::fmt::{Display, Formatter, Result as FmtResult};
use strum_macros::{Display, FromRepr};

#[expect(private_interfaces)]
pub enum Url<'a> {
	Charset(Charset),

	Filters {
		tags: Tags,
		status: Status,
		sort_by: Sort,
		page: i32,
		content_rating: ContentRating,
		viewing_permission: ViewingPermission,
	},

	Abs {
		path: &'a str,
	},

	Manga {
		id: &'a str,
	},

	Search {
		query: SearchQuery,
	},

	Uncensored {
		index: Index,
	},

	LastUpdated {
		query: LastUpdatedQuery,
	},

	Chart {
		page: i32,
	},

	Random,

	/// https://boylove.cc/home/api/chapter_list/tp/{manga_id}-0-0-10
	ChapterList(String),

	/// https://boylove.cc/home/book/capter/id/{chapter_id}
	Chapter(&'a str),

	/// https://boylove.cc/home/auth/login/type/login.html
	SignInPage,

	/// https://boylove.cc/home/auth/login.html
	SignIn,
}

#[derive(Default, Display)]
pub enum Charset {
	#[strum(to_string = "S")]
	Simplified,

	#[default]
	#[strum(to_string = "T")]
	Traditional,
}

pub struct Index {
	pub page: i32,
}

impl Display for Index {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		#[expect(clippy::arithmetic_side_effects)]
		let index = self.page - 1;

		write!(f, "{index}")
	}
}

pub struct LastUpdatedQuery {
	pub page: i32,
}

impl Display for LastUpdatedQuery {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let mut query = QueryParameters::new();

		query.push_encoded("widx", Some("11"));

		let page = self.page;
		let index = Index { page }.to_string();
		query.push_encoded("page", Some(&index));

		write!(f, "{query}")
	}
}

pub const DOMAIN: &str = "https://boylove.cc";
pub const MANGA_PATH: &str = "index/id/";
pub const CHAPTER_PATH: &str = "capter/id/";

pub trait DefaultRequest {
	fn default_headers(self) -> Self;
}

impl DefaultRequest for Request {
	fn default_headers(self) -> Self {
		self.header("Referer", DOMAIN).header(
			"User-Agent",
			"Mozilla/5.0 (iPhone; CPU iPhone OS 17_6 like Mac OS X) \
			 AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Mobile/15E148 Safari/604.1",
		)
	}
}

impl Url<'_> {
	pub fn get(self) -> Request {
		Request::get(self.to_string()).default_headers()
	}

	pub fn post<S: AsRef<str>>(self, data: S) -> Request {
		Request::post(self.to_string())
			.body(data.as_ref())
			.default_headers()
	}
}

impl From<Url<'_>> for String {
	fn from(url: Url) -> Self {
		url.to_string()
	}
}

impl From<(Vec<Filter>, i32)> for Url<'_> {
	fn from((filters, page): (Vec<Filter>, i32)) -> Self {
		let mut tags = Vec::new();

		macro_rules! init {
			($($filter:ident, $Filter:ident);+) => {
				$(let mut $filter = $Filter::default();)+
			};
		}
		init!(
			status, Status;
			content_rating, ContentRating;
			viewing_permission, ViewingPermission
		);

		for filter in filters {
			#[expect(clippy::wildcard_enum_match_arm)]
			match filter.kind {
				FilterType::Select => {
					macro_rules! get_filter {
						($Filter:ident) => {
							filter
								.value
								.as_int()
								.ok()
								.and_then(|i| {
									#[expect(
										clippy::cast_sign_loss,
										clippy::cast_possible_truncation
									)]
									$Filter::from_repr(i as _)
								})
								.unwrap_or_default()
						};
					}
					match filter.name.as_str() {
						"連載情形" => status = get_filter!(Status),
						"內容分級" => content_rating = get_filter!(ContentRating),
						"閱覽權限" => viewing_permission = get_filter!(ViewingPermission),
						_ => continue,
					}
				}

				FilterType::Title => {
					let keyword = match filter.value.as_string() {
						Ok(str_ref) => str_ref.read(),
						Err(_) => continue,
					};
					let query = SearchQuery { keyword, page };

					return Self::Search { query };
				}

				FilterType::Genre => {
					let is_not_checked = filter.value.as_int().unwrap_or(-1) != 1;
					if is_not_checked {
						continue;
					}

					tags.push(filter.name);
				}

				_ => continue,
			}
		}

		let sort_by = Sort::default();

		Self::Filters {
			tags: Tags(tags),
			status,
			sort_by,
			page,
			content_rating,
			viewing_permission,
		}
	}
}

impl<'a> Display for Url<'a> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let api_path = format!("{}/home/api/", DOMAIN);
		let html_path = format!("{}/home/book/", DOMAIN);
		let auth_path = format!("{}/home/auth/", DOMAIN);

		match self {
			Self::Charset(charset) => write!(f, "{DOMAIN}/home/user/to{charset}.html"),

			Self::Filters {
				tags,
				status,
				sort_by,
				page,
				content_rating,
				viewing_permission,
			} => write!(
				f,
				"{DOMAIN}/home/api/cate/tp/\
				 1-{tags}-{status}-{sort_by}-{page}-{content_rating}-1-{viewing_permission}",
			),

			Self::Abs { path } => write!(f, "{DOMAIN}{path}"),

			Self::Manga { id } => write!(f, "{DOMAIN}/home/book/index/id/{id}"),

			Self::Search { query } => write!(f, "{DOMAIN}/home/api/searchk?{query}"),

			Self::Uncensored { index } => {
				write!(f, "{DOMAIN}/home/api/getpage/tp/1-recommend-{index}")
			}

			Self::LastUpdated { query } => {
				write!(f, "{DOMAIN}/home/Api/getDailyUpdate.html?{query}")
			}

			Self::Chart { page } => {
				write!(f, "{DOMAIN}/home/index/pages/w/topestmh/page/{page}.html")
			}

			Self::Random => write!(f, "{DOMAIN}/home/Api/getCnxh.html"),

			Self::ChapterList(manga_id) => {
				write!(f, "{}chapter_list/tp/{}-0-0-10", api_path, manga_id)
			}

			Self::Chapter(chapter_id) => write!(f, "{}{}{}", html_path, CHAPTER_PATH, chapter_id),

			Self::SignInPage => write!(f, "{}login/type/login.html", auth_path),

			Self::SignIn => write!(f, "{}login.html", auth_path),
		}
	}
}

#[derive(Default, Display, FromRepr)]
enum Status {
	#[default]
	#[strum(to_string = "2")]
	All,

	#[strum(to_string = "0")]
	Ongoing,

	#[strum(to_string = "1")]
	Completed,
}

#[derive(Default, Display)]
enum Sort {
	#[default]
	#[strum(to_string = "1")]
	LastUpdated,
}

#[derive(Default, Display, FromRepr)]
enum ContentRating {
	#[default]
	#[strum(to_string = "0")]
	All,

	#[strum(to_string = "1")]
	Safe,

	#[strum(to_string = "2")]
	Nsfw,
}

#[derive(Default, Display, FromRepr)]
enum ViewingPermission {
	#[default]
	#[strum(to_string = "2")]
	All,

	#[strum(to_string = "0")]
	Basic,

	#[strum(to_string = "1")]
	Vip,
}

struct Tags(Vec<String>);

impl Display for Tags {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		if self.0.is_empty() {
			return write!(f, "0");
		}

		let tags_str = self
			.0
			.iter()
			.map(encode_uri_component)
			.collect::<Vec<_>>()
			.join("+");

		write!(f, "{tags_str}")
	}
}

struct SearchQuery {
	keyword: String,
	page: i32,
}

impl Display for SearchQuery {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let mut query = QueryParameters::new();
		query.push("keyword", Some(&self.keyword));
		query.push_encoded("type", Some("1"));
		query.push_encoded("pageNo", Some(&self.page.to_string()));

		write!(f, "{query}")
	}
}
