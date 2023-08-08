#![no_std]
use aidoku::{
	error::Result,
	helpers::{
		substring::Substring,
		uri::{encode_uri, QueryParameters},
	},
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, Filter,
	FilterType::{Check, Sort, Title},
	Manga,
	MangaContentRating::Nsfw,
	MangaPageResult,
	MangaStatus::*,
	Page,
};

extern crate alloc;
use alloc::string::ToString;

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

const DOMAIN: &str = "https://myreadingmanga.info";

/// Safari on iOS 16.4
const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.4 Mobile/15E148 Safari/604.1";

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

		let cover_url = encode_uri(manga_node.select("img").attr("src").read());

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
			nsfw: Nsfw,
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
	let mut status_vec: Vec<String> = Vec::new();
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
			match is_status {
				true => status_vec.push(tag),
				false => categories.push(tag),
			}
		}
	}

	let status = if status_vec.contains(&"Completed".to_string()) {
		Completed
	} else if status_vec.contains(&"Discontinued".to_string())
		|| status_vec.contains(&"Dropped".to_string())
	{
		Cancelled
	} else if status_vec.contains(&"Hiatus".to_string()) {
		Hiatus
	} else if status_vec.contains(&"Ongoing".to_string()) {
		Ongoing
	} else {
		Unknown
	};

	Ok(Manga {
		id: manga_id,
		title: manga_title,
		author: artists_str.clone(),
		artist: artists_str,
		description,
		url: manga_url,
		categories,
		status,
		nsfw: Nsfw,
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
	let mut pages = manga_html.select("a.post-page-numbers").array().len();
	if pages == 0 {
		pages = 1;
	}
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

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let chapter_url = Url::Chapter(&manga_id, &chapter_id).to_string();
	let chapter_html = request_get(&chapter_url).html()?;

	let mut pages = Vec::<Page>::new();
	let page_nodes = chapter_html.select("img[decoding=async][src^=https]");
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

fn get_filtered_url(filters: Vec<Filter>, page: i32) -> Result<String> {
	let mut query = QueryParameters::new();
	query.push_encoded("wpsolr_page", Some(page.to_string().as_str()));

	let mut sort_by_index = 1;
	let mut filters_vec = Vec::<(String, String)>::new();

	for filter in filters {
		match filter.kind {
			Check => {
				let is_not_checked = filter.value.as_int().unwrap_or(-1) != 1;
				if is_not_checked {
					continue;
				}

				let filter_type = filter.object.get("id").as_string()?.read();
				filters_vec.push((filter_type, filter.name));
			}

			Sort => {
				let obj = filter.value.as_object()?;
				sort_by_index = obj.get("index").as_int().unwrap_or(1) as u8;
			}

			Title => {
				let search_str = filter.value.as_string()?.read();
				query.push("wpsolr_q", Some(&search_str));

				query.push_encoded("wpsolr_sort", Some(SORT[0]));

				return Ok(Url::Search(query).to_string());
			}

			_ => continue,
		}
	}

	query.push_encoded("wpsolr_sort", Some(SORT[sort_by_index as usize]));
	filters_vec
		.iter()
		.enumerate()
		.for_each(|(filter_index, (filter_type, filter_value))| {
			query.push(
				format!("wpsolr_fq[{}]", filter_index),
				Some(format!("{}:{}", filter_type, filter_value)),
			)
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
		.map_or(String::new(), |value| value.replace('[', ""))
}

impl core::fmt::Display for Url<'_> {
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

trait Parser {
	/// Returns [`None`], or the text of the Node (if [`Ok`]).
	fn get_is_ok_text(self) -> Option<String>;
}

impl Parser for aidoku::std::ValueRef {
	fn get_is_ok_text(self) -> Option<String> {
		self.as_node().map_or(None, |node| Some(node.text().read()))
	}
}
