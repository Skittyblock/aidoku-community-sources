use aidoku::{
	error::Result,
	helpers::{substring::Substring, uri::QueryParameters},
	prelude::format,
	std::{html::Node, net::Request, String, Vec},
	Chapter, Filter, FilterType, Manga, MangaPageResult, MangaStatus, Page,
};

extern crate alloc;
use alloc::string::ToString;

pub const BASE_URL: &str = "https://mangabz.com/";
pub const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_3_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36";
const GENRE: [u8; 10] = [0, 31, 26, 1, 2, 25, 11, 17, 15, 34];
const SORT: [u8; 2] = [10, 2];

fn extract_chapter_number(title: &str) -> Option<f32> {
	let keywords = ["话", "話", "章", "回", "卷"];
	for &kw in &keywords {
		if let Some(pos) = title.rfind(kw) {
			let before = &title[..pos];
			let mut start = pos;
			while start > 0 && (before.as_bytes()[start - 1].is_ascii_digit() || before.as_bytes()[start - 1] == b'.') {
				start -= 1;
			}
			if start < pos {
				let num_str = &before[start..];
				if let Ok(num) = num_str.parse::<f32>() {
					return Some(num);
				}
			}
		}
	}
	None
}

pub fn get_filtered_url(filters: Vec<Filter>, page: i32) -> String {
	let mut is_searching = false;

	let mut genre = 0;
	let mut status = 0;
	let mut sort = 10;

	let mut query = QueryParameters::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					is_searching = true;
					query.push("title", Some(filter_value.read().as_str()));
				}
			}
			FilterType::Select => {
				let index = filter.value.as_int().unwrap_or(0) as usize;
				match filter.name.as_str() {
					"題材" => genre = GENRE[index],
					"狀態" => status = index as u8,
					_ => continue,
				}
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int().unwrap_or(0) as usize;
				sort = SORT[index];
			}
			_ => continue,
		}
	}

	let mut url = String::from(BASE_URL);
	if is_searching {
		query.push("page", Some(page.to_string().as_str()));
		url.push_str(format!("search?{}", query).as_str());
	} else {
		url.push_str(format!("manga-list-{}-{}-{}-p{}/", genre, status, sort, page).as_str());
	}
	url
}

pub fn request_get(url: String) -> Request {
	Request::get(url)
		.header("Referer", BASE_URL)
		.header("User-Agent", USER_AGENT)
}

pub fn get_manga_list(html: Node) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	for item in html.select("div.mh-item").array() {
		let manga_item = item.as_node()?;
		let title_node = manga_item.select("h2.title > a");

		let id = title_node
			.attr("href")
			.read()
			.replace('/', "")
			.replace("bz", "");
		let cover = manga_item.select("img.mh-cover").attr("src").read();
		let title = title_node.attr("title").read();
		let url = format!("{}{}bz/", BASE_URL, id);

		let status_str = manga_item.select("span").text().read();
		let status = match status_str.as_str() {
			"最新" => MangaStatus::Ongoing,
			"完結" => MangaStatus::Completed,
			_ => MangaStatus::Unknown,
		};

		mangas.push(Manga {
			id,
			cover,
			title,
			url,
			status,
			..Default::default()
		});
	}

	let has_more = !html
		.select("div.page-pagination a:contains(>)")
		.array()
		.is_empty();

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn get_manga_details(html: Node, id: String) -> Result<Manga> {
	let manga_info = html.select("p.detail-info-tip");

	let cover = html.select("img.detail-info-cover").attr("src").read();
	let title = html.select("p.detail-info-title").text().read();

	let mut artists: Vec<String> = Vec::new();
	for item in manga_info.select("span:contains(作者) > a").array() {
		let artist_str = item.as_node()?.text().read();
		artists.push(artist_str);
	}
	let artist = artists.join("、");

	let description = html.select("p.detail-info-content").text().read();
	let url = format!("{}{}bz/", BASE_URL, id);

	let status_str = manga_info
		.select("span:contains(狀態) > span")
		.text()
		.read();
	let status = match status_str.as_str() {
		"連載中" => MangaStatus::Ongoing,
		"已完結" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};

	let mut categories: Vec<String> = Vec::new();
	for item in manga_info.select("span.item").array() {
		let genre = item.as_node()?.text().read();
		categories.push(genre);
	}

	Ok(Manga {
		id,
		cover,
		title,
		author: artist.clone(),
		artist,
		description,
		url,
		categories,
		status,
		..Default::default()
	})
}

pub fn get_chapter_list(html: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	let mut index = 1.0;

	for item in html.select("a.detail-list-form-item").array().rev() {
		let chapter_item = item.as_node()?;

		let id = chapter_item.attr("href").read().replace(['/', 'm'], "");
		let title = chapter_item.text().read();
		let clean_title = title.split('（').next().unwrap_or(&title).trim();
		let chapter_or_volume = extract_chapter_number(&title).unwrap_or(index);
		let (ch, vo) = if clean_title.ends_with('卷') {
			(-1.0, chapter_or_volume)
		} else {
			(chapter_or_volume, -1.0)
		};
		let url = format!("{}m{}/", BASE_URL, id);

		let scanlator = if vo > -1.0 {
			"单行本".to_string()
		} else {
			"默认".to_string()
		};

		chapters.insert(
			0,
			Chapter {
				id,
				title,
				volume: vo,
				chapter: ch,
				url,
				lang: String::from("zh"),
				scanlator,
				..Default::default()
			},
		);
		index += 1.0;
	}

	Ok(chapters)
}

pub fn get_page_list(url: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	let mut page = 1;
	let mut last_url = String::new();

	loop {
		let content = request_get(format!("{}{}", url, page)).string()?;
		let urls = decode(content);
		for url in urls.clone() {
			if url == last_url {
				break;
			}
			last_url = url.clone();

			let index = page - 1;
			pages.push(Page {
				index,
				url,
				..Default::default()
			});
			page += 1;
		}
		if urls.len() == 1 {
			break;
		}
	}

	Ok(pages)
}

fn decode(encoded: String) -> Vec<String> {
	let mut urls: Vec<String> = Vec::new();

	let packed = encoded
		.substring_after("return p;}")
		.expect("packed")
		.to_string();

	let k: Vec<&str> = packed
		.substring_after(",\'")
		.expect("k")
		.substring_before("\'.")
		.expect("k")
		.split('|')
		.collect();

	let chapter = decoded_with_k(
		packed
			.substring_after("=\"")
			.expect("base")
			.substring_before("\";")
			.expect("base")
			.to_string(),
		k.clone(),
	);
	let query = decoded_with_k(
		packed
			.substring_before_last("\\\'")
			.expect("query")
			.substring_after_last("\\\'")
			.expect("query")
			.to_string(),
		k.clone(),
	);

	let pages: Vec<&str> = packed
		.substring_after("=[")
		.expect("pages")
		.substring_before("];")
		.expect("pages")
		.split(',')
		.collect();
	for item in pages {
		let page = decoded_with_k(item.replace('\"', "").to_string(), k.clone());
		urls.push(format!("{}{}{}", chapter, page, query));
	}

	urls
}

fn decoded_with_k(encoded: String, k: Vec<&str>) -> String {
	let mut decoded = String::new();

	for char in encoded.chars() {
		let mut str;

		if char.is_digit(36) {
			str = if char.is_ascii_uppercase() {
				k[(char as usize) - 29].to_string()
			} else {
				k[char.to_digit(36).expect("base 36") as usize].to_string()
			};
			if str.is_empty() {
				str = char.to_string();
			}
		} else {
			str = char.to_string();
		}
		decoded.push_str(str.as_str());
	}

	decoded
}
