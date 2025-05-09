use crate::constants::{BASE_API_URL, BASE_URL, CHAPTER_PAGE_SIZE, PAGE_SIZE, USER_AGENT};
use crate::dto::{BaseMangaItem, FetchMangaInfo};
use crate::parser::{parse_branches, parse_chapters, parse_manga_fetch_info};
use aidoku::helpers::uri::QueryParameters;
use aidoku::std::defaults::defaults_get;
use aidoku::std::net::Request;
use aidoku::{error::Result, std::ObjectRef, Chapter, Filter, FilterType};
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::{format, string::String};

// debugging
/*use aidoku::prelude::println;
macro_rules! debug {
	($($arg:tt)*) => {{
		println!("ru.remanga:: {}:{}: {}", file!(), line!(), format!($($arg)*))
	}};
}
pub(crate) use debug;*/

pub fn get_manga_title(item: &BaseMangaItem) -> String {
	let main_title = defaults_get("main_title")
		.and_then(|value| value.as_bool())
		.unwrap_or(false);

	if main_title {
		item.main_name.clone()
	} else {
		item.secondary_name.clone()
	}
}

pub fn build_url_to_title(dir: &String) -> String {
	format!("{BASE_URL}/manga/{dir}/")
}

pub fn build_url_to_chapter(id: &String, dir: &String) -> String {
	format!("{BASE_URL}/manga/{dir}/{id}")
}

pub fn build_url_to_cover(url: String) -> String {
	build_url(String::from(BASE_URL), url)
}

pub fn build_api_title_url(dir: String) -> String {
	build_url(String::from(BASE_API_URL), format!("/titles/{dir}/"))
}

fn build_api_search_url(search_query: String, page: i32) -> Result<String> {
	let mut query = QueryParameters::default();
	query.push(String::from("count"), Some(PAGE_SIZE.to_string()));
	query.push("field", Some("titles"));
	query.push(String::from("page"), Some(page.to_string()));
	query.push(String::from("query"), Some(search_query));

	Ok(build_url_with_query(
		String::from(BASE_API_URL),
		String::from("/search/"),
		query,
	))
}

pub fn build_api_filter_url(filters: Vec<Filter>, page: i32) -> Result<String> {
	let mut query = QueryParameters::default();
	query.push(String::from("count"), Some(PAGE_SIZE.to_string()));
	query.push(String::from("page"), Some(page.to_string()));

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(search_query) = filter.value.as_string() {
					return build_api_search_url(search_query.read(), page);
				}
			}
			FilterType::Genre => {
				let info = filter
					.object
					.get("id")
					.as_string()
					.map(|x| x.read())
					.unwrap_or_default();

				if let Some((type_id, id)) = info.split_once('|') {
					let q_key = match type_id {
						"0" => "genres",
						"1" => "categories",
						"2" => "types",
						_ => continue,
					};
					match filter.value.as_int().unwrap_or(-1) {
						0 => query.push(format!("exclude_{q_key}"), Some(String::from(id))),
						1 => query.push(q_key, Some(id)),
						_ => continue,
					}
				}
			}
			FilterType::Check => {
				let info = filter
					.object
					.get("id")
					.as_string()
					.map(|x| x.read())
					.unwrap_or_default();

				if let Some((type_id, id)) = info.split_once("|") {
					let q_key = match type_id {
						"0" => "status",
						"1" => "translate_status",
						_ => continue,
					};
					query.push(q_key, Some(id));
				}
			}
			FilterType::Select => {
				let age_limit = filter.value.as_int().unwrap_or(0) - 1;
				if age_limit > -1 {
					query.push(String::from("age_limit"), Some(age_limit.to_string()));
				}
			}
			FilterType::Sort => {
				if let Ok(value) = filter.value.as_object() {
					let ascending = value.get("ascending").as_bool().unwrap_or(false);
					let mut order = match value.get("index").as_int().unwrap_or(-1) {
						0 => String::from("id"),
						1 => String::from("count_chapters"),
						2 => String::from("chapter_date"),
						// 3 is skipped cuz it's default one
						4 => String::from("avg_rating"),
						5 => String::from("count_rating"),
						6 => String::from("votes"),
						7 => String::from("views"),
						_ => String::from("rating"),
					};

					if !ascending {
						order.insert(0, '-');
					}

					query.push(String::from("ordering"), Some(order));
				}
			}
			_ => continue,
		}
	}

	Ok(build_url_with_query(
		String::from(BASE_API_URL),
		String::from("/search/catalog/"),
		query,
	))
}

pub fn build_api_chapters_url(branch_id: String, page: i32) -> String {
	let mut query = QueryParameters::default();
	query.push(String::from("count"), Some(CHAPTER_PAGE_SIZE.to_string()));
	query.push(String::from("branch_id"), Some(branch_id.clone()));
	query.push("ordering", Some("-index"));
	query.push(String::from("page"), Some(page.to_string()));
	query.push("user_data", Some("1"));

	build_url_with_query(
		String::from(BASE_API_URL),
		String::from("/titles/chapters/"),
		query,
	)
}

pub fn build_api_chapter_pages_url(chapter_id: String) -> String {
	format!("{BASE_API_URL}/titles/chapters/{chapter_id}/")
}

fn build_url(base_url: String, url: String) -> String {
	if !url.starts_with('/') {
		return url;
	}

	format!("{base_url}{url}")
}

fn build_url_with_query(base_url: String, url: String, query: QueryParameters) -> String {
	if !url.starts_with('/') {
		return url;
	}

	format!("{base_url}{url}?{query}")
}

pub fn fetch_manga_info(dir: String) -> Result<FetchMangaInfo> {
	fetch_json(build_api_title_url(dir.clone()))
		.and_then(|obj| obj.get("branches").as_array())
		.and_then(parse_branches)
		.and_then(|branches| parse_manga_fetch_info(format!("{branches}:{dir}")))
}

pub fn fetch_all_chapters(id: String) -> Result<Vec<Chapter>> {
	parse_manga_fetch_info(id).and_then(|info| {
		let mut chapters = Vec::new();

		if info.branches.is_empty() {
			return Ok(Vec::new());
		}

		for branch in info.branches {
			let mut page = 1;
			loop {
				let obj = fetch_json(build_api_chapters_url(branch.clone(), page))?;

				let list = obj
					.get("results")
					.as_array()
					.and_then(|results| parse_chapters(info.dir.clone(), results))?;
				if list.is_empty() {
					break;
				}

				chapters.extend(list);

				let no_next = obj.get("next").as_string().unwrap_or_default().is_empty();
				if no_next {
					break;
				}

				page += 1;
			}
		}

		chapters.sort_by(|a, b| b.index.cmp(&a.index));

		Ok(chapters.iter().map(|x| x.item.clone()).collect())
	})
}

pub fn fetch_json<T: AsRef<str>>(url: T) -> Result<ObjectRef> {
	let token = defaults_get("token")?
		.as_string()
		.map(|x| x.read())
		.map(|x| {
			if x.is_empty() {
				x
			} else {
				format!("Bearer {x}")
			}
		})
		.unwrap_or_default();

	let request = Request::get(&url)
		.header("User-Agent", USER_AGENT)
		.header("Referer", BASE_URL);

	if token.is_empty() {
		request.json()?.as_object()
	} else {
		request
			.header(String::from("Authorization"), token)
			.json()?
			.as_object()
	}
}
