use aidoku::{
	error::Result,
	prelude::*,
	std::Vec,
	std::{html::Node, json, String},
	Chapter, Manga, MangaContentRating, MangaStatus, MangaViewer, Page,
};
use alloc::{borrow::ToOwned, string::ToString};

pub fn urlencode(string: &String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if (b'a'..=b'z').contains(&curr)
			|| (b'A'..=b'Z').contains(&curr)
			|| (b'0'..=b'9').contains(&curr)
		{
			result.push(curr);
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}

	String::from_utf8(result).unwrap_or_default()
}

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

pub fn create_advanced_search_body(
	manga_title: Option<String>,
	artist_name: Option<String>,
	status: Option<i64>,
	tag_search_mode: Option<String>,
	tag_list: Option<Vec<i64>>,
) -> String {
	let mut form_body = format!("cmd_wpm_wgt_mng_sch_sbm=Search&txt_wpm_wgt_mng_sch_nme=&cmd_wpm_pag_mng_sch_sbm=&txt_wpm_pag_mng_sch_nme={}&txt_wpm_pag_mng_sch_ats={}&rad_wpm_pag_mng_sch_sts={}&rad_wpm_pag_mng_sch_tag_mde={}",
		urlencode(&manga_title.unwrap_or_default()),
        urlencode(&artist_name.unwrap_or_default()),
        &status.unwrap_or_default(),
        &tag_search_mode.unwrap_or_default()
	);

	if let Some(tag_list) = tag_list {
		for tag in tag_list.iter() {
			form_body.push_str(format!("&chk_wpm_pag_mng_sch_mng_tag_inc[]={}", tag).as_str());
		}
	}

	form_body
}

pub fn genre_id_from_filter(str: &str) -> i64 {
	let genre_id = str.split('_').last().unwrap_or_default();
	genre_id.parse::<i64>().unwrap_or_default()
}

pub fn clean_cover_url(str: &str) -> String {
	// /cdn-cgi/image/format=auto/https://img1.hentaicdn.com/hentai/cover/_S38878.jpg?x63162
	let mut url = str.to_owned();
	url.replace_range(0..url.find("https://").unwrap_or_default(), "");
	url
}

pub fn parse_chapter_number(str: &str) -> f32 {
	let chapter_number = str.split('/').nth_back(1).unwrap_or_default();
	chapter_number.parse::<f32>().unwrap_or_default()
}

pub fn change_page(str: &str, page: i32) -> String {
	let mut url = str.to_owned();
	let page_str = url.split('/').nth_back(1).unwrap_or_default();
	url.replace_range(url.len() - page_str.len().., &i32_to_string(page));
	url
}

pub fn get_manga_id(str: &str) -> String {
	let url = str.to_owned();

	let manga_id = url.split('/').nth_back(1).unwrap_or_default();
	manga_id.to_string()
}

pub fn parse_search(html: &Node) -> Vec<Manga> {
	let mut manga_arr: Vec<Manga> = Vec::new();
	for result in html
		.select(".block-content.row .book-grid-item-container")
		.array()
	{
		let mut manga_url = String::new();
		let mut manga_id = String::new();
		let mut cover = String::new();
		let mut title = String::new();

		if let Ok(result_node) = result.as_node() {
			let cover_url = result_node
				.select("picture img")
				.first()
				.attr("data-src")
				.to_string();
			cover = clean_cover_url(&cover_url);

			let manga_url_node = result_node.select("a");
			manga_url = manga_url_node.attr("href").to_string();
			manga_id = get_manga_id(&manga_url);
			title = manga_url_node.select("span.title-text").text().to_string();
		}

		manga_arr.push(Manga {
			id: manga_id,
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: manga_url,
			categories: Vec::new(),
			status: MangaStatus::Completed,
			nsfw: MangaContentRating::Nsfw,
			viewer: MangaViewer::Rtl,
		});
	}

	manga_arr
}

pub fn parse_manga(id: String, html: &Node) -> Result<Manga> {
	let mut author = String::new();
	let mut artist = String::new();
	let mut categories = Vec::new();
	let mut status = MangaStatus::Unknown;

	let title_node = html.select(".content .block-header h3.block-title a");
	let title = title_node.own_text().read();

	let cover = html.select(".img-container a img").attr("src").to_string();

	let url = format!("https://hentai2read.com/{}/", id);

	for item in html
		.select(".list.list-simple-mini li.text-primary")
		.array()
	{
		let li = item.as_node().unwrap();
		let key = li.select("b").text().read();
		match key.to_lowercase().as_str() {
			"status" => {
				let tag_button = li.select("a.tagButton").text().read().to_lowercase();
				status = match tag_button.as_str() {
					"ongoing" => MangaStatus::Ongoing,
					"completed" => MangaStatus::Completed,
					_ => MangaStatus::Unknown,
				}
			}
			"author" => {
				author = li.select("a.tagButton").text().read();
			}
			"artist" => {
				artist = li.select("a.tagButton").text().read();
			}
			"content" | "categories" => {
				for category in li.select("a.tagButton").array() {
					let category = category.as_node().unwrap();
					categories.push(category.own_text().read());
				}
			}

			_ => continue,
		}
	}

	Ok(Manga {
		id,
		title,
		author,
		artist,
		description: String::new(),
		cover,
		url,
		categories,
		status,
		nsfw: MangaContentRating::Nsfw,
		viewer: MangaViewer::Rtl,
	})
}

pub fn parse_chapter_list(html: &Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	for link in html.select("ul.nav-chapters > li > div.media > a").array() {
		let node = link.as_node().unwrap();
		let title = node.own_text().read();
		let chapter = parse_chapter_number(&node.attr("href").to_string());
		let url = String::new();

		chapters.push(Chapter {
			id: format!("{}", chapter),
			title,
			volume: -1.0,
			chapter,
			date_updated: -1.0,
			scanlator: String::new(),
			url,
			lang: String::from("en"),
		})
	}

	Ok(chapters)
}

fn between_string(s: &str, start: &str, end: &str) -> String {
	let start_bytes = s.find(start).unwrap() + start.len();
	let end_bytes = s.find(end).unwrap();
	let mut result = String::new();
	for (_, c) in s[start_bytes..end_bytes].chars().enumerate() {
		result.push(c);
	}
	result
}

pub fn parse_page_list(html: &Node) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	let mut script = String::new();

	for el in html.select("script").array() {
		let node = el.as_node().unwrap();
		let text = node.html().read();
		if text.contains("gData") {
			script = text;
			break;
		}
	}
	// between { and }

	let obj = between_string(&script, "'images' : [", "\"]");
	let arr_str = format!("[{}\"]", obj);
	let dataref = json::parse(arr_str).unwrap();
	let arr = dataref.as_array().unwrap();

	for (i, item) in arr.enumerate() {
		let img_path = item.as_string().unwrap().read();
		let img_url = format!("https://static.hentaicdn.com/hentai/{}", img_path);
		pages.push(Page {
			index: i as i32,
			url: img_url.to_string(),
			base64: String::new(),
			text: String::new(),
		})
	}

	Ok(pages)
}
