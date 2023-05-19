use aidoku::{
	error::Result,
	helpers::{substring::Substring, uri::QueryParameters},
	prelude::{format, println},
	std::{html::Node, net::Request, String, Vec},
	Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
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
	let mut sort_by = SORT_BY[1];

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					is_searching = true;
					query.push("wpsolr_q", Some(filter_value.read().as_str()));
				}
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int().unwrap_or(1) as usize;
				sort_by = SORT_BY[index];
			}
			_ => continue,
		}
	}

	if is_searching {
		query.push("wpsolr_sort", Some(SORT_BY[0]));
	} else {
		query.push("wpsolr_sort", Some(sort_by));
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

	for manga_item in html.select("div.results-by-facets > div").array() {
		let manga_item = manga_item.as_node()?;
		let title_node = manga_item.select("a");

		let title = title_node.text().read();
		if !title.starts_with('[') {
			continue;
		}

		let url = title_node.attr("href").read();
		let id = url.replace(BASE_URL, "").replace('/', "");
		let cover = manga_item.select("img").attr("src").read();
		let artist = title
			.substring_before(']')
			.expect("artist &str")
			.replace('[', "");
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

	html.close();

	Ok(MangaPageResult { manga, has_more })
}

pub fn get_manga_details(html: Node, id: String) -> Result<Manga> {
	let title = html.select("h1.entry-title").text().read();
	let artist = title
		.substring_before("]")
		.expect("[artist -> &str")
		.replace('[', "");
	let url = format!("{}{}/", BASE_URL, id);

	let mut description: Vec<String> = Vec::new();
	for p in html.select("div.entry-content > p").array() {
		let p = p.as_node()?.text().read().to_lowercase();
		if p.starts_with("chapter") {
			break;
		}
		description.push(p);
	}
	let description = description.join("\n");

	let mut categories: Vec<String> = Vec::new();
	let mut status_vec: Vec<String> = Vec::new();

	for span in html.select("footer.entry-footer span").array() {
		let span = span.as_node()?;
		let span_text = span.own_text().read();

		let mut is_status = false;
		if span_text.starts_with("Status:") {
			is_status = true;
		} else if span_text.starts_with("Scanlation by:") {
			continue;
		}

		for tag in span.select("a").array() {
			let tag = tag.as_node()?.text().read();
			if is_status {
				status_vec.push(tag);
			} else {
				categories.push(tag);
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
