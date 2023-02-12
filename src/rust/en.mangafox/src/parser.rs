use aidoku::{
	error::Result,
	helpers::substring::Substring,
	prelude::*,
	std::{html::Node, String, Vec},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

use crate::unpacker;

extern crate alloc;
use alloc::string::ToString;

pub fn parse_directory(html: Node) -> Result<MangaPageResult> {
	let mut result: Vec<Manga> = Vec::new();
	let has_more: bool = !is_last_page(html.clone());

	for page in html.select("ul.line li").array() {
		let obj = page.as_node().expect("html array not an array of nodes");

		let id = obj
			.select("a")
			.attr("href")
			.read()
			.replace("/manga/", "")
			.replace('/', "");
		let title = obj.select("a").attr("title").read();
		let cover = obj.select("a img").attr("src").read();

		result.push(Manga {
			id,
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: String::new(),
			categories: Vec::new(),
			status: MangaStatus::Unknown,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Rtl,
		});
	}
	Ok(MangaPageResult {
		manga: result,
		has_more,
	})
}

pub fn parse_manga(obj: Node, id: String) -> Result<Manga> {
	let cover = obj.select(".detail-info-cover-img").attr("data-src").read();
	let title = obj
		.select("span.detail-info-right-title-font")
		.text()
		.read();
	let author = obj.select("p.detail-info-right-say a").text().read();
	let description = obj.select("p.fullcontent").text().read();

	let url = String::from("https://www.fanfox.net/manga/") + &id;

	let mut nsfw: MangaContentRating = MangaContentRating::Safe;
	let mut categories: Vec<String> = Vec::new();
	obj.select(".detail-info-right-tag-list a")
		.array()
		.for_each(|tag_html| {
			let tag = String::from(
				tag_html
					.as_node()
					.expect("Array of tags wasn't nodes. Wow 0^0.")
					.text()
					.read()
					.trim(),
			);
			if tag == "Ecchi" || tag == "Mature" || tag == "Smut" || tag == "Adult" {
				nsfw = MangaContentRating::Nsfw;
			}
			categories.push(tag);
		});

	let status_str = obj
		.select(".detail-info-right-title-tip")
		.text()
		.read()
		.to_lowercase();
	let status = if status_str.contains("Ongoing") {
		MangaStatus::Ongoing
	} else if status_str.contains("Completed") {
		MangaStatus::Completed
	} else {
		MangaStatus::Unknown
	};

	// let viewer = match type_str.as_str() {
	// 	"manga" => MangaViewer::Rtl,
	// 	"manhwa" => MangaViewer::Scroll,
	// 	_ => MangaViewer::Rtl,
	// };

	Ok(Manga {
		id,
		cover,
		title,
		author,
		artist: String::new(),
		description,
		url,
		categories,
		status,
		nsfw,
		viewer: MangaViewer::Rtl,
	})
}

pub fn parse_chapters(obj: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	for item in obj.select(".detail-main-list li").array() {
		let obj = item.as_node().expect("");
		let id = obj
			.select("a")
			.attr("href")
			.read()
			.replace("/manga/", "")
			.replace("/1.html", "");

		let url = String::from("https://www.fanfox.net/manga/") + &id;

		// parse title
		let mut title = String::new();
		let title_str = obj.select(".title3").text().read();
		let split = title_str.as_str().split('-');
		let vec = split.collect::<Vec<&str>>();
		if vec.len() > 1 {
			let (_, rest) = vec.split_first().unwrap();
			title = rest.join("-")
		}

		let mut volume = -1.0;
		let mut chapter = -1.0;

		// parse volume and chapter
		let split = id.as_str().split('/');
		let vec = split.collect::<Vec<&str>>();
		for item in vec {
			let f_char = &item.chars().next().unwrap();
			match f_char {
				'v' => {
					volume = String::from(item)
						.trim_start_matches('v')
						.parse::<f32>()
						.unwrap_or(-1.0)
				}
				'c' => {
					chapter = String::from(item)
						.trim_start_matches('c')
						.parse::<f32>()
						.unwrap_or(-1.0)
				}
				_ => continue,
			}
		}

		let date_updated = obj
			.select(".title2")
			.text()
			.0
			.as_date("MMM dd,yyyy", None, None)
			.unwrap_or(-1.0);

		chapters.push(Chapter {
			id,
			title,
			volume,
			chapter,
			date_updated,
			scanlator: String::new(),
			url,
			lang: String::from("en"),
		});
	}
	Ok(chapters)
}

pub fn get_page_list(html: Node) -> Result<Vec<Page>> {
	let mut eval_script = String::new();
	for item in html.select("script").array() {
		let script = item.as_node().expect("");
		let body = script.html().read();
		if body.contains("eval(function(p,a,c,k,e,d){") {
			eval_script = body;
		}
	}

	let evaluated = unpacker::unpack(eval_script);

	let page_img_str = evaluated
		.substring_after("var newImgs=[\"//")
		.unwrap()
		.substring_before("\"];var newImginfos=")
		.unwrap()
		.to_string();

	let str_page_arr = page_img_str
		.as_str()
		.split("\",\"//")
		.collect::<Vec<&str>>();

	let mut pages: Vec<Page> = Vec::new();
	for (index, string) in str_page_arr.iter().enumerate() {
		let url = format!("https://{}", string);
		pages.push(Page {
			index: index as i32,
			url: url.to_string(),
			base64: String::new(),
			text: String::new(),
		});
	}

	Ok(pages)
}

pub fn get_filtered_url(filters: Vec<Filter>, page: i32) -> String {
	let mut is_searching = false;
	let mut search_query = String::new();
	let mut genre_query = String::new();
	let mut url = String::from("https://fanfox.net");

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					search_query.push_str("&name=");
					search_query.push_str(urlencode(filter_value.read().to_lowercase()).as_str());
					is_searching = true;
				}
			}
			FilterType::Genre => {
				if let Ok(filter_id) = filter.object.get("id").as_string() {
					genre_query.push_str(filter_id.read().as_str());
					is_searching = true;
				}
			}
			FilterType::Select => {
				if filter.name == "Language" {
					search_query.push_str("&type=");
					if filter.value.as_int().unwrap_or(-1) > 0 {
						search_query
							.push_str(&i32_to_string(filter.value.as_int().unwrap_or(-1) as i32));
						is_searching = true;
					}
				}
				if filter.name == "Rating" {
					search_query.push_str("&rating_method=eq&rating=");
					if filter.value.as_int().unwrap_or(-1) > 0 {
						search_query
							.push_str(&i32_to_string(filter.value.as_int().unwrap_or(-1) as i32));
						is_searching = true;
					}
				}
				if filter.name == "Completed" {
					search_query.push_str("&st=");
					if filter.value.as_int().unwrap_or(-1) > 0 {
						search_query
							.push_str(&i32_to_string(filter.value.as_int().unwrap_or(-1) as i32));
						is_searching = true;
					}
				}
			}
			_ => continue,
		}
	}

	if is_searching {
		url.push_str("/search?page=");
		url.push_str(&page.to_string());
		url.push_str(&search_query);
	} else {
		url.push_str("/directory/");
		url.push_str(&page.to_string());
		url.push_str(".html?rating")
	}
	url
}

// pub fn parse_incoming_url(url: String) -> String {
// 	// https://mangapill.com/manga/6290/one-piece-pirate-recipes
// 	// https://mangapill.com/chapters/6290-10006000/one-piece-pirate-recipes-chapter-6

// 	let split = url.as_str().split('/');
// 	let vec = split.collect::<Vec<&str>>();
// 	let mut manga_id = String::from("/manga/");

// 	if url.contains("/chapters/") {
// 		let split = vec[vec.len() - 2].split('-');
// 		let ch_vec = split.collect::<Vec<&str>>();
// 		manga_id.push_str(ch_vec[0]);
// 	} else {
// 		manga_id.push_str(vec[vec.len() - 2]);
// 	}
// 	manga_id.push('/');
// 	manga_id.push_str(vec[vec.len() - 1]);
// 	manga_id
// }

pub fn is_last_page(html: Node) -> bool {
	let length = &html.select("div.pager-list-left a").array().len();
	for (index, page) in html.select("div.pager-list-left a").array().enumerate() {
		let page_node = page.as_node().expect("Failed to get page node");
		let href = page_node.attr("href").read();
		if index == length - 1 && href == "javascript:void(0)" {
			return true;
		}
	}
	false
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

//  https://fanfox.net/search?page=4&type=&rating_method=eq&rating=&st=&name=merc
//  https://fanfox.net/search?title=&genres=&nogenres=&sort=&stype=1&name=merc&type=0&author_method=cw&author=&artist_method=cw&artist=&rating_method=eq&rating=&released_method=eq&released=&st=0
