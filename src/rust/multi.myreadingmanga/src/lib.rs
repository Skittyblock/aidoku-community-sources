#![no_std]
use aidoku::{
	error::Result,
	helpers::{substring::Substring, uri::QueryParameters},
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus, Page,
};

extern crate alloc;
use alloc::string::ToString;

const DOMAIN: &str = "https://myreadingmanga.info";
const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.4 Mobile/15E148 Safari/604.1";

fn get(url: String) -> Request {
	Request::get(url)
		.header("Referer", DOMAIN)
		.header("User-Agent", USER_AGENT)
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let manga_list_url = get_filtered_url(filters, page)?;
	let manga_list_html = get(manga_list_url).html()?;

	let mut manga: Vec<Manga> = Vec::new();

	for value in manga_list_html
		.select("div.results-by-facets > div")
		.array()
	{
		let manga_node = value.as_node()?;
		let title_node = manga_node.select("a");

		let title = title_node.text().read();

		// Skip videos
		// There are some exceptions
		let is_video = !title.starts_with('[');
		if is_video {
			continue;
		}

		let url = title_node.attr("href").read();

		let id = url.replace(DOMAIN, "").replace('/', "");

		let cover = manga_node.select("img").attr("src").read();

		let artist = match title.substring_before(']') {
			Some(artists_str) => artists_str.replace('[', ""),
			None => String::new(),
		};

		let description = manga_node.select("div.p_content").text().read();

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

	let pages = manga_list_html.select("a.paginate").array().len() as i32;
	let has_more = page < pages;

	Ok(MangaPageResult { manga, has_more })
}

pub fn get_filtered_url(filters: Vec<Filter>, page: i32) -> Result<String> {
	const SORT_BY: [&str; 4] = [
		"sort_by_relevancy_desc",
		"sort_by_date_desc",
		"sort_by_date_asc",
		"sort_by_random",
	];

	let mut url = format!("{}/search/?", DOMAIN);

	let mut query = QueryParameters::new();
	query.push("wpsolr_page", Some(page.to_string().as_str()));

	let mut sort_by_index = 1;
	let mut filters_vec: Vec<(String, String)> = Vec::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				let search_str = filter.value.as_string()?.read();
				query.push("wpsolr_q", Some(search_str.as_str()));

				query.push("wpsolr_sort", Some(SORT_BY[0]));

				url.push_str(format!("{}", query).as_str());
				return Ok(url);
			}

			FilterType::Sort => {
				let object = filter.value.as_object()?;
				sort_by_index = object.get("index").as_int().unwrap_or(1) as u8;
			}

			FilterType::Check => {
				let checked = filter.value.as_int().unwrap_or(-1) == 1;
				if !checked {
					continue;
				}

				let filter_type = filter.object.get("id").as_string()?.read();
				filters_vec.push((filter_type, filter.name));
			}

			_ => continue,
		}
	}

	query.push("wpsolr_sort", Some(SORT_BY[sort_by_index as usize]));
	for (index, value) in filters_vec.iter().enumerate() {
		let (filter_type, filter_value) = value;
		query.push(
			format!("wpsolr_fq[{}]", index),
			Some(format!("{}:{}", filter_type, filter_value)),
		);
	}
	url.push_str(format!("{}", query).as_str());

	Ok(url)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{}/{}/", DOMAIN, id);

	let manga_html = get(url.clone()).html()?;

	let title = manga_html.select("h1.entry-title").text().read();

	let artist = match title.substring_before("]") {
		Some(artists_str) => artists_str.replace('[', ""),
		None => String::new(),
	};

	let mut description_vec: Vec<String> = Vec::new();
	for value in manga_html.select("div.entry-content > p").array() {
		let p_text = value.as_node()?.text().read();

		let is_not_description = p_text
			.replace(|char: char| !char.is_ascii_alphanumeric(), "")
			.to_lowercase()
			.starts_with("chapter");
		if is_not_description {
			break;
		}

		description_vec.push(p_text);
	}
	let description = description_vec.join("\n").trim().to_string();

	let mut categories: Vec<String> = Vec::new();
	let mut status_vec: Vec<String> = Vec::new();
	for value in manga_html.select("footer.entry-footer span").array() {
		let span_node = value.as_node()?;
		let span_text = span_node.own_text().read();

		let mut is_status = false;
		if span_text.starts_with("Status:") {
			is_status = true;
		} else if span_text.starts_with("Scanlation by:") {
			continue;
		}

		for a_value in span_node.select("a").array() {
			let tag = a_value.as_node()?.text().read();
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

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let manga_url = format!("{}/{}/", DOMAIN, id);
	let manga_html = get(manga_url.clone()).html()?;

	let mut scanlators_vec: Vec<String> = Vec::new();
	for value in manga_html
		.select("footer.entry-footer span.entry-terms:contains(Scanlation by:) > a")
		.array()
	{
		let scanlator_str = value.as_node()?.text().read();
		scanlators_vec.push(scanlator_str);
	}
	let scanlator = scanlators_vec.join(", ");

	let mut chapters: Vec<Chapter> = Vec::new();

	let mut pages = manga_html.select("a.post-page-numbers").array().len();
	if pages == 0 {
		pages = 1;
	}

	for index in 1..=pages {
		let url = format!("{}{}/", manga_url, index);

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

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let chapter_url = format!("{}/{}/{}/", DOMAIN, manga_id, chapter_id);
	let page_html = get(chapter_url).html()?;

	let mut pages: Vec<Page> = Vec::new();

	for (index, value) in page_html
		.select("img[decoding=async][src^=https]")
		.array()
		.enumerate()
	{
		let url = value.as_node()?.attr("src").read();

		pages.push(Page {
			index: index as i32,
			url,
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
