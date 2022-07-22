use alloc::string::ToString;
use alloc::vec;

use aidoku::{
	error::Result, prelude::*, std::html::Node, std::String, std::Vec, Chapter, Manga,
	MangaContentRating, MangaStatus, MangaViewer, Page,
};
use chrono::NaiveDate;

use crate::BASE_URL;

const REPLACE_STRINGS: [&str; 6] = [":", "-", "/", "(", ")", "%"];

pub fn parse_manga_list(html: Node) -> Vec<Manga> {
	let mut result: Vec<Manga> = Vec::new();

	for page in html
		.select(".ranking_lst.popular")
		.array()
		.next()
		.unwrap()
		.as_node()
		.select("ul > li")
		.array()
	{
		let obj = page.as_node();
		if &result.len() >= &10 {
			break;
		}

		let url = obj.select("a").attr("href").read();
		let id = substr_after(&url, "webtoons.com/en/").to_string();
		let cover = obj
			.select("img")
			.attr("src")
			.read()
			.replace("a92", "crop540_540");
		let title = obj.select(".subj").first().text().read().trim().to_string();
		let author = obj
			.select(".author")
			.first()
			.text()
			.read()
			.trim()
			.to_string();

		if id.is_empty() || title.is_empty() {
			continue;
		}

		result.push(Manga {
			id,
			cover,
			title,
			author,
			url,
			artist: String::new(),
			description: String::new(),
			categories: Vec::new(),
			status: MangaStatus::Ongoing,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll,
		});
	}

	result
}

pub fn parse_search(html: &Node, challenge: bool) -> Vec<Manga> {
	let mut result: Vec<Manga> = Vec::new();
	let selector_class = if challenge {
		".challenge_lst"
	} else {
		".card_lst"
	};

	for page in html.select(selector_class).select("li").array() {
		let obj = page.as_node();

		let cover = obj.select("img").attr("src").read();
		let title = obj.select(".subj").first().text().read().trim().to_string();
		let author = obj
			.select(".author")
			.first()
			.text()
			.read()
			.trim()
			.to_string();
		let genre = obj.select(".genre").text().read().trim().to_string();

		let mut url_title = title.replace(" ", "-").to_lowercase();
		for replace_string in REPLACE_STRINGS.iter() {
			url_title = url_title.replace(replace_string, "");
		}

		let url_prefix = if challenge {
			"challenge".to_string()
		} else {
			genre.replace(" ", "-").to_lowercase()
		};
		let id_num =
			substr_after(obj.select("a").attr("href").read().as_str(), "titleNo=").to_string();

		let url = format!(
			"{}/en/{}/{}/list?title_no={}",
			BASE_URL, url_prefix, url_title, id_num
		);
		let id = substr_after(&url, "webtoons.com/en/").to_string();

		result.push(Manga {
			id,
			cover,
			title,
			author,
			url,
			artist: String::new(),
			description: String::new(),
			categories: Vec::new(),
			status: MangaStatus::Ongoing,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll,
		});
	}

	result
}

pub fn parse_manga(obj: Node, id: String) -> Result<Manga> {
	let url = format!("{}/en/{}", BASE_URL, &id);
	let title = obj.select(".subj").first().text().read().trim().to_string();
	let description = obj.select("p.summary").text().read().trim().to_string();
	let mut cover = obj
		.select(".background_pic")
		.select("img")
		.attr("src")
		.read();
	if cover.is_empty() {
		cover = obj
			.select(".detail_chal_pic")
			.select("img")
			.attr("src")
			.read();
	}
	let author = obj
		.select(".author")
		.first()
		.text()
		.read()
		.trim()
		.to_string();
	let genre = vec![obj.select(".genre").text().read().trim().to_string()];

	Ok(Manga {
		id,
		cover,
		title,
		author,
		description,
		url,
		categories: genre,
		artist: String::new(),
		status: MangaStatus::Ongoing,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Scroll,
	})
}

pub fn get_chapter_list(obj: Node, manga_id: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	for chapter in obj.select("#_episodeList").select("li").array() {
		let obj = chapter.as_node();

		let id = obj.attr("data-episode-no").read().to_string();
		if id.is_empty() {
			continue;
		}

		let chapter = id.parse::<f32>().unwrap();

		// The mobile website sucks so we need to manually replace some chars
		let title = obj
			.select(".sub_title span")
			.text()
			.read()
			.trim()
			.to_string()
			.replace("&amp;", "&")
			.replace("&quot;", "\"")
			.replace("&#039;", "'")
			.replace("&lt;", "<")
			.replace("&gt;", ">")
			.replace("&nbsp;", " ");

		let date_str = obj.select(".date").text().read().to_string();
		let date = NaiveDate::parse_from_str(&date_str, "%b %d, %Y")
			.unwrap()
			.and_hms(0, 0, 0);

		chapters.push(Chapter {
			id: format!("{}|{}", manga_id, id),
			title,
			chapter,
			volume: 1.0,
			date_updated: date.timestamp() as f64,
			scanlator: String::new(),
			url: String::new(),
			lang: String::from("en"),
		});
	}

	Ok(chapters)
}

pub fn get_page_list(obj: Node) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	let mut i = 0;
	for page in obj.select(".viewer_lst").select("img").array() {
		let obj = page.as_node();
		let url = obj.attr("data-url").read();

		pages.push(Page {
			index: i as i32,
			url,
			base64: String::new(),
			text: String::new(),
		});
		i += 1;
	}
	Ok(pages)
}

pub fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if (b'a' <= curr && curr <= b'z')
			|| (b'A' <= curr && curr <= b'Z')
			|| (b'0' <= curr && curr <= b'9')
		{
			result.push(curr);
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}

	String::from_utf8(result).unwrap_or(String::new())
}

fn substr_after(string: &str, needle: &str) -> String {
	let index = string.find(needle).unwrap();
	let substr = &string[index + needle.len()..];
	return substr.to_string();
}
