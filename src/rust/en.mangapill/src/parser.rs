use aidoku::{
	error::Result, prelude::*, std::html::Node, std::String, std::Vec, Chapter, Filter, FilterType,
	Manga, MangaContentRating, MangaStatus, MangaViewer, Page,
};

pub const BASE_URL: &str = "https://www.mangapill.com";
pub const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_3_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36";

pub fn parse_recents(html: Node, result: &mut Vec<Manga>) {
	for page in html.select("div.grid div:not([class])").array() {
		let obj = page.as_node().expect("node array");

		let id = obj.select("a.text-secondary").attr("href").read();
		let title = obj.select("a.text-secondary").text().read();
		let img = obj.select("a figure img").attr("data-src").read();

		result.push(Manga {
			id,
			cover: img,
			title,
			..Default::default()
		});
	}
}

pub fn parse_search(html: Node, result: &mut Vec<Manga>) {
	for page in html.select(".grid.gap-3 div").array() {
		let obj = page.as_node().expect("node array");

		let id = obj.select("a").attr("href").read();
		let title = obj.select("div a ").text().read();
		let img = obj.select("a figure img").attr("data-src").read();

		if !id.is_empty() && !title.is_empty() && !img.is_empty() {
			result.push(Manga {
				id,
				cover: img,
				title,
				..Default::default()
			});
		}
	}
}

pub fn parse_manga(obj: Node, id: String) -> Result<Manga> {
	let title = obj.select(".lazy").attr("alt").read();
	let cover = obj.select(".lazy").attr("data-src").read();
	let description = obj.select(".text-sm.text--secondary").text().read();
	let type_str = obj
		.select(".grid.grid-cols-1.gap-3.mb-3 div:first-child div")
		.text()
		.read()
		.to_lowercase();
	let status_str = obj
		.select(".grid.grid-cols-1.gap-3.mb-3 div:nth-child(2) div:nth-child(2)")
		.text()
		.read()
		.to_lowercase();

	let url = format!("https://www.mangapill.com{}", &id);

	let mut categories: Vec<String> = Vec::new();
	obj.select("a[href*=genre]")
		.array()
		.for_each(|tag| categories.push(tag.as_node().expect("node array").text().read()));

	let status = if status_str.contains("publishing") {
		MangaStatus::Ongoing
	} else if status_str.contains("finished") {
		MangaStatus::Completed
	} else {
		MangaStatus::Unknown
	};

	let nsfw = if obj
		.select(".alert-warning")
		.text()
		.read()
		.contains("Mature")
	{
		MangaContentRating::Nsfw
	} else if categories.contains(&String::from("Ecchi")) {
		MangaContentRating::Suggestive
	} else {
		MangaContentRating::Safe
	};

	let viewer = match type_str.as_str() {
		"manga" => MangaViewer::Rtl,
		"manhwa" => MangaViewer::Scroll,
		_ => MangaViewer::Rtl,
	};

	Ok(Manga {
		id,
		cover,
		title,
		description,
		url,
		categories,
		status,
		nsfw,
		viewer,
		..Default::default()
	})
}

pub fn get_chaper_list(obj: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	for chapter in obj.select(".p-1").array() {
		let obj = chapter.as_node().expect("node array");
		let id = obj.attr("href").read();
		let url = format!("https://www.mangapill.com{}", &id);
		if id == "Read Chapters" {
			continue;
		}

		let split = id.as_str().split('-');
		let vec = split.collect::<Vec<&str>>();
		let chap_num = vec[vec.len() - 1].parse().unwrap();

		chapters.push(Chapter {
			id,
			chapter: chap_num,
			url,
			lang: String::from("en"),
			..Default::default()
		});
	}
	Ok(chapters)
}

pub fn get_page_list(obj: Node) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	for (i, page) in obj.select("picture img").array().enumerate() {
		let obj = page.as_node().expect("node array");
		let url = obj.attr("data-src").read();

		pages.push(Page {
			index: i as i32,
			url,
			..Default::default()
		});
	}
	Ok(pages)
}

pub fn get_filtered_url(filters: Vec<Filter>, page: i32, url: &mut String) {
	let mut is_searching = false;
	let mut query = String::new();
	let mut search_string = String::new();
	url.push_str("https://mangapill.com");

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					// filter_value.read().to_lowercase();
					search_string.push_str(urlencode(filter_value.read().to_lowercase()).as_str());
					is_searching = true;
				}
			}
			FilterType::Genre => {
				query.push_str("&genre=");
				query.push_str(&urlencode(filter.name.as_str().to_lowercase()));
				is_searching = true;
			}
			FilterType::Select => {
				if filter.name.as_str() == "Type" {
					query.push_str("&type=");
					match filter.value.as_int().unwrap_or(-1) {
						0 => query.push_str(""),
						1 => query.push_str("manga"),
						2 => query.push_str("novel"),
						3 => query.push_str("one-shot"),
						4 => query.push_str("doujinshi"),
						5 => query.push_str("manhwa"),
						6 => query.push_str("manhua"),
						7 => query.push_str("oel"),
						_ => continue,
					}
					if filter.value.as_int().unwrap_or(-1) > 0 {
						is_searching = true;
					}
				}
				if filter.name.as_str() == "Status" {
					query.push_str("&status=");
					match filter.value.as_int().unwrap_or(-1) {
						0 => query.push_str(""),
						1 => query.push_str("publishing"),
						2 => query.push_str("finished"),
						3 => query.push_str("on+haitus"),
						4 => query.push_str("doujinshi"),
						5 => query.push_str("discontinued"),
						_ => continue,
					}
					if filter.value.as_int().unwrap_or(-1) > 0 {
						is_searching = true;
					}
				}
			}
			_ => continue,
		}
	}

	if is_searching {
		url.push_str("/search?");
		url.push_str("q=");
		url.push_str(&search_string);
		url.push_str(&query);
		url.push_str("&page=");
		url.push_str(&i32_to_string(page));
	}
}

pub fn parse_incoming_url(url: String) -> String {
	// https://mangapill.com/manga/6290/one-piece-pirate-recipes
	// https://mangapill.com/chapters/6290-10006000/one-piece-pirate-recipes-chapter-6

	let split = url.as_str().split('/');
	let vec = split.collect::<Vec<&str>>();
	let mut manga_id = String::from("/manga/");

	if url.contains("/chapters/") {
		let split = vec[vec.len() - 2].split('-');
		let ch_vec = split.collect::<Vec<&str>>();
		manga_id.push_str(ch_vec[0]);
	} else {
		manga_id.push_str(vec[vec.len() - 2]);
	}
	manga_id.push('/');
	manga_id.push_str(vec[vec.len() - 1]);
	manga_id
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
