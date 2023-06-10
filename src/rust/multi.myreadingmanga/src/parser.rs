use aidoku::{
	error::Result,
	helpers::{substring::Substring, uri::QueryParameters},
	prelude::format,
	std::{html::Node, net::Request, String, Vec},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus, Page,
};

extern crate alloc;
use alloc::string::ToString;

pub const BASE_URL: &str = "https://myreadingmanga.info/";
pub const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.4 Mobile/15E148 Safari/604.1";
const SORT_BY: [&str; 4] = [
	"sort_by_relevancy_desc",
	"sort_by_date_desc",
	"sort_by_date_asc",
	"sort_by_random",
];

pub fn get_filtered_url(filters: Vec<Filter>, page: i32) -> String {
	let mut query = QueryParameters::new();

	let mut is_searching = false;
	let mut sort_by_index = 1;
	let mut filter_vec: Vec<(String, String)> = Vec::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(value) = filter.value.as_string() {
					is_searching = true;
					query.push("wpsolr_q", Some(value.read().as_str()));
				}
			}
			FilterType::Sort => {
				if let Ok(value) = filter.value.as_object() {
					sort_by_index = value.get("index").as_int().unwrap_or(1) as u8;
				}
			}
			FilterType::Check => {
				let checked = filter.value.as_int().unwrap_or(-1) == 1;
				if !checked {
					continue;
				}

				if let Ok(id) = filter.object.get("id").as_string() {
					let filter_type = id.read();
					let filter_name = filter.name.as_str().to_string();
					filter_vec.push((filter_type, filter_name));
				}
			}
			_ => continue,
		}
	}

	match is_searching {
		true => query.push("wpsolr_sort", Some(SORT_BY[0])),
		false => {
			query.push("wpsolr_sort", Some(SORT_BY[sort_by_index as usize]));
			for (index, item) in filter_vec.iter().enumerate() {
				let (filter_type, filter_value) = item;
				query.push(
					format!("wpsolr_fq[{}]", index),
					Some(format!("{}:{}", filter_type, filter_value)),
				);
			}
		}
	}
	query.push("wpsolr_page", Some(page.to_string().as_str()));

	let url = format!("{}search/?{}", BASE_URL, query);
	url
}

pub fn request_get(url: String) -> Request {
	Request::get(url)
		.header("Referer", BASE_URL)
		.header("User-Agent", USER_AGENT)
}

pub fn get_manga_list(html: Node, page: i32) -> Result<MangaPageResult> {
	let mut manga: Vec<Manga> = Vec::new();

	for item in html.select("div.results-by-facets > div").array() {
		let manga_item = item.as_node()?;
		let title_node = manga_item.select("a");

		// Skip videos
		// There are some exceptions
		let title = title_node.text().read();
		if !title.starts_with('[') {
			continue;
		}

		let url = title_node.attr("href").read();
		let id = url.replace(BASE_URL, "").replace('/', "");
		let cover = manga_item.select("img").attr("src").read();
		let artist = match title.substring_before(']') {
			Some(name) => name.replace('[', ""),
			None => String::new(),
		};
		let description = manga_item.select("div.p_content").text().read();

		manga.push(Manga {
			id,
			cover,
			title,
			author: artist.clone(),
			artist,
			description,
			url,
			nsfw: MangaContentRating::Nsfw,
			..Default::default()
		});
	}

	let pages = html.select("a.paginate").array().len() as i32;
	let has_more = page < pages;

	Ok(MangaPageResult { manga, has_more })
}

pub fn get_manga_details(html: Node, id: String) -> Result<Manga> {
	let title = html.select("h1.entry-title").text().read();
	let artist = match title.substring_before("]") {
		Some(name) => name.replace('[', ""),
		None => String::new(),
	};
	let url = format!("{}{}/", BASE_URL, id);

	let mut description_vec: Vec<String> = Vec::new();
	for item in html.select("div.entry-content > p").array() {
		let p = item.as_node()?.text().read();
		let is_description = p
			.replace(|char: char| !char.is_ascii_alphanumeric(), "")
			.to_lowercase()
			.starts_with("chapter");
		if is_description {
			break;
		}
		description_vec.push(p);
	}
	let description = description_vec.join("\n");

	let mut categories: Vec<String> = Vec::new();
	let mut status_vec: Vec<String> = Vec::new();

	for item in html.select("footer.entry-footer span").array() {
		let span = item.as_node()?;
		let span_text = span.own_text().read();

		let mut is_status = false;
		if span_text.starts_with("Status:") {
			is_status = true;
		} else if span_text.starts_with("Scanlation by:") {
			continue;
		}

		for item in span.select("a").array() {
			let tag = item.as_node()?.text().read();
			match is_status {
				true => status_vec.push(tag),
				false => categories.push(tag),
			}
		}
	}

	let status = if status_vec.contains(&"Completed".to_string()) {
		MangaStatus::Completed
	} else if status_vec.contains(&"Discontinued".to_string())
		|| status_vec.contains(&"Dropped".to_string())
	{
		MangaStatus::Cancelled
	} else if status_vec.contains(&"Hiatus".to_string()) {
		MangaStatus::Hiatus
	} else if status_vec.contains(&"Ongoing".to_string()) {
		MangaStatus::Ongoing
	} else {
		MangaStatus::Unknown
	};

	Ok(Manga {
		id,
		title,
		author: artist.clone(),
		artist,
		description,
		url,
		categories,
		status,
		nsfw: MangaContentRating::Nsfw,
		..Default::default()
	})
}

pub fn get_chapter_list(html: Node, manga_id: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	let mut scanlator_vec: Vec<String> = Vec::new();
	for item in html
		.select("footer.entry-footer span.entry-terms:contains(Scanlation by:) > a")
		.array()
	{
		let scanlator_str = item.as_node()?.text().read();
		scanlator_vec.push(scanlator_str);
	}
	let scanlator = scanlator_vec.join(", ");

	let mut pages = html.select("a.post-page-numbers").array().len();
	if pages == 0 {
		pages = 1;
	}
	for index in 1..=pages {
		let mut url = format!("{}{}/", BASE_URL, manga_id);
		if index > 1 {
			url.push_str(format!("{}/", index).as_str());
		}

		chapters.insert(
			0,
			Chapter {
				id: index.to_string(),
				chapter: index as f32,
				scanlator: scanlator.clone(),
				url,
				..Default::default()
			},
		);
	}

	Ok(chapters)
}

pub fn get_page_list(html: Node) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	for (index, item) in html
		.select("img[decoding=async][src^=https]")
		.array()
		.enumerate()
	{
		let url = item.as_node()?.attr("src").read();

		pages.push(Page {
			index: index as i32,
			url,
			..Default::default()
		});
	}

	Ok(pages)
}
