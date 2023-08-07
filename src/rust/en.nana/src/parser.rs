use aidoku::{
	error::Result,
	prelude::*,
	std::html::Node,
	std::Vec,
	std::{ObjectRef, String},
	Filter, FilterType, Manga, MangaContentRating, MangaStatus, MangaViewer, Page,
};

const BASE_URL: &str = "https://nana.my.id";

pub fn parse_search(html: Node) -> Vec<Manga> {
	let mut result = Vec::new();
	for page in html.select("#thumbs_container > .id1").array() {
		if let Ok(obj) = page.as_node() {
			let a = obj.select(".id3 > a");
			let id: String = a.attr("href").read().split('/').last().unwrap().into();

			let url = format!("{}/reader/{}", BASE_URL, &id);
			let title = a.attr("title").read();
			let author = a
				.select("img")
				.attr("alt")
				.read()
				.replace(&format!("{} by ", title), "");

			let img = a.select("img").attr("src").read();
			let img_url = if img.starts_with('/') {
				format!("{}{}", BASE_URL, img)
			} else {
				img
			};

			let mut categories: Vec<String> = Vec::new();
			obj.select(".id4 > .tags > span")
				.array()
				.for_each(|tag| categories.push(tag.as_node().unwrap().text().read()));

			if !id.is_empty() && !title.is_empty() && !img_url.is_empty() {
				result.push(Manga {
					id,
					cover: img_url,
					title,
					author,
					artist: String::new(),
					description: String::new(),
					url,
					categories,
					status: MangaStatus::Completed,
					nsfw: MangaContentRating::Nsfw,
					viewer: MangaViewer::Scroll,
				});
			}
		}
	}

	result
}

pub fn get_page_list(obj: ObjectRef) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	for (i, page) in (obj.get("pages").as_array()?).enumerate() {
		let cleanid: String = page
			.as_string()?
			.read()
			.replace("thumbnails", "pages")
			.chars()
			.skip(1)
			.collect();
		let url = format!("{}{}", BASE_URL, cleanid);

		pages.push(Page {
			index: i as i32,
			url,
			base64: String::new(),
			text: String::new(),
		});
	}

	Ok(pages)
}

pub fn get_filtered_url(filters: Vec<Filter>, page: i32) -> String {
	let mut query = String::new();
	let mut ascending = false;

	let mut included_tags: Vec<String> = Vec::new();
	let mut excluded_tags: Vec<String> = Vec::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(value) = filter.value.as_string() {
					query = value.read();
				}
			}
			FilterType::Genre => {
				if let Ok(tag_id) = filter.object.get("id").as_string() {
					match filter.value.as_int().unwrap_or(-1) {
						0 => excluded_tags.push(tag_id.read()),
						1 => included_tags.push(tag_id.read()),
						_ => continue,
					}
				}
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				ascending = value.get("ascending").as_bool().unwrap_or(false);
			}
			_ => continue,
		}
	}

	build_search_url(query, included_tags, excluded_tags, ascending, page)
}

pub fn build_search_url(
	query: String,
	included_tags: Vec<String>,
	excluded_tags: Vec<String>,
	ascending: bool,
	page: i32,
) -> String {
	let mut url = String::new();

	url.push_str(format!("https://nana.my.id/").as_str());
	url.push('?');

	url.push_str(
		format!(
			"p={}&sort={}&q={}",
			i32_to_string(page),
			if ascending { "asc" } else { "desc" },
			urlencode(query).as_str()
		)
		.as_str(),
	);
	let mut query_params = String::new();
	if !included_tags.is_empty() {
		for tag in included_tags {
			query_params.push_str(format!("+\"{}\"", tag).as_str());
		}
	}
	if !excluded_tags.is_empty() {
		for tag in excluded_tags {
			query_params.push_str(format!("-\"{}\"", tag).as_str());
		}
	}

	url.push_str(urlencode(query_params).as_str());

	url
}

// HELPER FUNCTIONS

pub fn i32_to_string(mut integer: i32) -> String {
	if integer == 0 {
		return String::from("0");
	}
	let mut string = String::with_capacity(11);
	let pos = if integer < 0 {
		string.insert(0, '-');
		1
	} else {
		0
	};
	while integer != 0 {
		let mut digit = integer % 10;
		if pos == 1 {
			digit *= -1;
		}
		string.insert(pos, char::from_u32((digit as u32) + ('0' as u32)).unwrap());
		integer /= 10;
	}
	string
}

pub fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if curr.is_ascii_lowercase() || curr.is_ascii_uppercase() || curr.is_ascii_digit() {
			result.push(curr);
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}

	String::from_utf8(result).unwrap_or_default()
}
