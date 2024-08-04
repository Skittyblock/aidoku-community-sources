use aidoku::{
	helpers::uri::{encode_uri_component, QueryParameters},
	prelude::format,
	std::{net::Request, String, Vec},
	Filter, FilterType,
};
use alloc::string::ToString as _;
use core::fmt::{Display, Formatter, Result as FmtResult};
use strum_macros::{Display, FromRepr};

#[expect(private_interfaces)]
#[derive(Display)]
#[strum(prefix = "https://boylove.cc")]
pub enum Url<'a> {
	#[strum(to_string = "")]
	Domain,

	#[strum(to_string = "/home/user/to{charset}.html")]
	Charset { charset: Charset },

	#[strum(
		to_string = "/home/api/cate/tp/1-{tags}-{status}-{sort_by}-{page}-{content_rating}-1-{viewing_permission}"
	)]
	Filters {
		tags: Tags,
		status: Status,
		sort_by: Sort,
		page: i32,
		content_rating: ContentRating,
		viewing_permission: ViewingPermission,
	},

	#[strum(to_string = "{path}")]
	Abs { path: &'a str },

	#[strum(to_string = "/home/book/index/id/{id}")]
	Manga { id: &'a str },

	#[strum(to_string = "/home/api/searchk?{query}")]
	Search { query: SearchQuery },

	#[strum(to_string = "/home/api/getpage/tp/1-recommend-{index}")]
	Uncensored { index: Index },

	#[strum(to_string = "/home/Api/getDailyUpdate.html?{query}")]
	LastUpdated { query: LastUpdatedQuery },

	#[strum(to_string = "/home/index/pages/w/topestmh/page/{page}.html")]
	Chart { page: i32 },

	#[strum(to_string = "/home/Api/getCnxh.html")]
	Random,

	#[strum(to_string = "/home/api/chapter_list/tp/{id}-0-0-10")]
	ChapterList { id: &'a str },

	#[strum(to_string = "/home/book/capter/id/{id}")]
	ChapterPage { id: &'a str },

	#[strum(to_string = "/chapter_view_template?{query}")]
	Chapter { query: ChapterQuery<'a> },
}

impl Url<'_> {
	pub fn get(&self) -> Request {
		Request::get(self.to_string()).default_headers()
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

pub struct ChapterQuery<'a> {
	pub id: &'a str,
}

impl Display for ChapterQuery<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let mut query = QueryParameters::new();
		query.push_encoded("id", Some(self.id));

		write!(f, "{query}")
	}
}

pub trait DefaultRequest {
	fn default_headers(self) -> Self;
}

impl DefaultRequest for Request {
	fn default_headers(self) -> Self {
		let referer = Url::Domain.to_string();
		self.header("Referer", &referer).header(
			"User-Agent",
			"Mozilla/5.0 (iPhone; CPU iPhone OS 17_6 like Mac OS X) \
			 AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Mobile/15E148 Safari/604.1",
		)
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
