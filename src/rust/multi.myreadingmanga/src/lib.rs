#![no_std]
extern crate alloc;

use aidoku::{
	error::Result,
	helpers::{
		substring::Substring,
		uri::{internal_encode_uri, QueryParameters},
	},
	prelude::*,
	std::{net::Request, String, ValueRef, Vec},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	Page,
};
use alloc::string::ToString;
use core::fmt::Display;

enum Url<'a> {
	/// https://myreadingmanga.info/search/?wpsolr_page={}&wpsolr_q={}&wpsolr_sort={}&wpsolr_fq\[{index}\]={filter_id}:{filter_name}
	///
	/// ---
	///
	/// `wpsolr_page`: Start from `1`
	///
	/// `wpsolr_q`: `search_str` ➡️ Should be percent-encoded
	///
	/// `wpsolr_sort`:
	///
	/// - `sort_by_relevancy_desc`: More relevant
	/// - `sort_by_date_desc`: Newest
	/// - `sort_by_date_asc`: Oldest
	/// - `sort_by_random`: Random
	///
	/// `wpsolr_fq`:
	///
	/// - `index`: Start from `0`
	/// - `filter_name` ➡️ Should be percent-encoded
	Search(QueryParameters),

	/// https://myreadingmanga.info/{manga_id}/
	Manga(&'a str),

	/// https://myreadingmanga.info/{manga_id}/{chapter_id}/
	Chapter(&'a str, &'a str),
}

impl Url<'_> {
	fn search(keyword: String, page: i32) -> Self {
		let mut query = QueryParameters::new();

		let encoded_keyword = keyword.percent_encode(true);
		query.push_encoded("wpsolr_q", Some(&encoded_keyword));

		query.push_encoded("wpsolr_sort", Some(SORT[0]));

		query.push_encoded("wpsolr_page", Some(&page.to_string()));

		Self::Search(query)
	}
}

const DOMAIN: &str = "https://myreadingmanga.info";

/// Chrome 128 on iOS 17.6
/// Apple iPhone
const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_6_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/128.0.6613.98 Mobile/15E148 Safari/604.1";

/// Sort by: \[More relevant, Newest, Oldest, Random\]
const SORT: [&str; 4] = [
	"sort_by_relevancy_desc",
	"sort_by_date_desc",
	"sort_by_date_asc",
	"sort_by_random",
];

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let manga_list_url = get_filtered_url(filters, page)?;
	let manga_list_html = request_get(&manga_list_url).html()?;

	let mut manga = Vec::<Manga>::new();
	let manga_nodes = manga_list_html.select("div.results-by-facets > div");
	for manga_value in manga_nodes.array() {
		let manga_node = manga_value.as_node()?;
		let title_node = manga_node.select("a");

		let manga_title = title_node.text().read();

		// ? Skip videos
		// ! There are some exceptions
		let is_video = !manga_title.starts_with('[');
		if is_video {
			continue;
		}

		let manga_url = title_node.attr("href").read();

		let manga_id = manga_url.replace(DOMAIN, "").replace('/', "");

		let cover_url = manga_node
			.select("img")
			.attr("src")
			.read()
			.replace("-200x280", "")
			.percent_encode(false);

		let artists_str = get_artists(&manga_title);

		let description = manga_node.select("div.p_content").text().read();

		manga.push(Manga {
			id: manga_id,
			cover: cover_url,
			title: manga_title,
			author: artists_str.clone(),
			artist: artists_str,
			description,
			url: manga_url,
			nsfw: MangaContentRating::Nsfw,
			..Default::default()
		});
	}

	let pages = manga_list_html.select("a.paginate").array().len() as i32;
	let has_more = page < pages;

	Ok(MangaPageResult { manga, has_more })
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let manga_url = Url::Manga(&manga_id).to_string();

	let manga_html = request_get(&manga_url).html()?;

	let manga_title = manga_html.select("h1.entry-title").text().read();

	let search_title_url = Url::search(manga_title.clone(), 1).to_string();
	let manga_selector = format!("div.results-by-facets > div:has(a[href*={manga_id}]) img");
	let cover = request_get(&search_title_url)
		.html()?
		.select(manga_selector)
		.attr("src")
		.read()
		.replace("-200x280", "");

	let artists_str = get_artists(&manga_title);

	let description = manga_html
		.select("div.entry-content > p")
		.array()
		.filter_map(Parser::get_is_ok_text)
		.take_while(|p_text| {
			!p_text
				.replace(|char: char| !char.is_ascii_alphanumeric(), "")
				.to_lowercase()
				.starts_with("chapter")
		})
		.collect::<Vec<String>>()
		.join("\n")
		.trim()
		.to_string();

	let mut categories = Vec::<String>::new();
	let (mut is_completed, mut is_cancelled, mut is_hiatus, mut is_ongoing) =
		(false, false, false, false);
	let span_nodes = manga_html.select("footer.entry-footer span");
	for span_value in span_nodes.array() {
		let span_node = span_value.as_node()?;
		let span_text = span_node.own_text().read();

		let mut is_status = false;
		if span_text.starts_with("Status:") {
			is_status = true;
		} else if span_text.starts_with("Scanlation by:") {
			continue;
		}

		let a_nodes = span_node.select("a");
		for a_value in a_nodes.array() {
			let tag = a_value.as_node()?.text().read();
			if is_status {
				match tag.as_str() {
					"Completed" => is_completed = true,
					"Discontinued" | "Dropped" => is_cancelled = true,
					"Hiatus" => is_hiatus = true,
					"Ongoing" => is_ongoing = true,
					_ => (),
				}
			} else {
				categories.push(tag);
			}
		}
	}

	let status = if is_completed {
		MangaStatus::Completed
	} else if is_cancelled {
		MangaStatus::Cancelled
	} else if is_hiatus {
		MangaStatus::Hiatus
	} else if is_ongoing {
		MangaStatus::Ongoing
	} else {
		MangaStatus::Unknown
	};

	Ok(Manga {
		id: manga_id,
		cover,
		title: manga_title,
		author: artists_str.clone(),
		artist: artists_str,
		description,
		url: manga_url,
		categories,
		status,
		nsfw: MangaContentRating::Nsfw,
		..Default::default()
	})
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let manga_url = Url::Manga(&manga_id).to_string();
	let manga_html = request_get(&manga_url).html()?;

	let scanlators_str = manga_html
		.select("footer.entry-footer span.entry-terms:contains(Scanlation by:) > a")
		.array()
		.filter_map(Parser::get_is_ok_text)
		.collect::<Vec<String>>()
		.join(", ");

	let mut chapters = Vec::<Chapter>::new();
	let pages = manga_html
		.select(".page-numbers:not(.next)")
		.last()
		.text()
		.read()
		.parse()
		.unwrap_or(1);
	for chapter_index in 1..=pages {
		let chapter_id = chapter_index.to_string();

		let chapter_num = chapter_index as f32;

		let chapter_url = Url::Chapter(&manga_id, &chapter_id).to_string();

		let chapter = Chapter {
			id: chapter_id,
			chapter: chapter_num,
			scanlator: scanlators_str.clone(),
			url: chapter_url,
			..Default::default()
		};
		chapters.insert(0, chapter);
	}
	if let Some(last_chapter) = chapters.first_mut() {
		last_chapter.date_updated =
			manga_html
				.select("time.entry-time")
				.text()
				.as_date("MMMM d, yyyy", None, None);
	}

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let chapter_url = Url::Chapter(&manga_id, &chapter_id).to_string();
	let chapter_html = request_get(&chapter_url).html()?;

	let mut pages = Vec::<Page>::new();
	let page_nodes = chapter_html.select("img.img-myreadingmanga");
	for (page_index, page_value) in page_nodes.array().enumerate() {
		let page_url = page_value.as_node()?.attr("src").read();

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

#[expect(clippy::needless_pass_by_value)]
#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let mut parts = url.split('/').skip(3);

	let Some(manga_id) = parts.next() else {
		return Ok(DeepLink::default());
	};

	let manga = get_manga_details(manga_id.into())?;

	let chapter = parts
		.next()
		.filter(|path| path.chars().all(|c| c.is_ascii_digit()))
		.map(|id| Chapter {
			id: id.into(),
			..Default::default()
		});

	Ok(DeepLink {
		manga: Some(manga),
		chapter,
	})
}

fn get_filtered_url(filters: Vec<Filter>, page: i32) -> Result<String> {
	let mut query = QueryParameters::new();
	query.push_encoded("wpsolr_page", Some(page.to_string().as_str()));

	let mut sort_by_index = 1;
	let mut filters_vec = Vec::<(String, String)>::new();

	for filter in filters {
		match filter.kind {
			FilterType::Check => {
				let is_not_checked = filter.value.as_int().unwrap_or(-1) != 1;
				if is_not_checked {
					continue;
				}

				let filter_type = filter.object.get("id").as_string()?.read();
				filters_vec.push((filter_type, filter.name));
			}

			FilterType::Sort => {
				let obj = filter.value.as_object()?;
				sort_by_index = obj.get("index").as_int().unwrap_or(1) as u8;
			}

			FilterType::Title => {
				let search_str = filter.value.as_string()?.read();

				return Ok(Url::search(search_str, page).to_string());
			}

			_ => continue,
		}
	}

	query.push_encoded("wpsolr_sort", Some(SORT[sort_by_index as usize]));
	filters_vec
		.iter()
		.enumerate()
		.for_each(|(filter_index, (filter_type, filter_value))| {
			query.push_encoded(
				format!("wpsolr_fq[{}]", filter_index).percent_encode(true),
				Some(format!("{}:{}", filter_type, filter_value).percent_encode(true)),
			);
		});

	Ok(Url::Search(query).to_string())
}

fn request_get(url: &str) -> Request {
	Request::get(url)
		.header("Referer", DOMAIN)
		.header("User-Agent", USER_AGENT)
}

fn get_artists(title: &str) -> String {
	title
		.substring_before(']')
		.map(|value| value.replace('[', ""))
		.unwrap_or_default()
}

impl Display for Url<'_> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Url::Search(query) => write!(f, "{}/search/?{}", DOMAIN, query),

			Url::Manga(manga_id) => write!(f, "{}/{}/", DOMAIN, manga_id),

			Url::Chapter(manga_id, chapter_id) => {
				write!(f, "{}{}/", Url::Manga(manga_id), chapter_id)
			}
		}
	}
}

trait UrlString {
	/// ' should be percent-encoded
	fn percent_encode(self, is_component: bool) -> String;
}

impl UrlString for String {
	fn percent_encode(self, is_component: bool) -> String {
		let char_set = "-_.!~*()".to_string() + if is_component { "" } else { ";,/?:@&=+$#" };

		internal_encode_uri(self, char_set)
	}
}

trait Parser {
	/// Returns [`None`], or the text of the Node (if [`Ok`]).
	fn get_is_ok_text(self) -> Option<String>;
}

impl Parser for ValueRef {
	fn get_is_ok_text(self) -> Option<String> {
		self.as_node().ok().map(|node| node.text().read())
	}
}
