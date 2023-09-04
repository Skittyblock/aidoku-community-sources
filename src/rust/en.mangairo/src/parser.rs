use aidoku::{
	error::Result, prelude::*, std::html::Node, std::String, std::Vec, Chapter, Filter, FilterType,
	Manga, MangaContentRating, MangaStatus, MangaViewer, Page, MangaPageResult,
};

pub const BASE_URL: &str = "https://w.mangairo.com";
pub const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_3_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36";

pub fn parse_recents(html: Node, result: &mut Vec<Manga>) {
	todo!()
}

pub fn parse_search(html: Node, result: &mut Vec<Manga>) {
	todo!()
}

pub fn parse_manga(obj: Node, id: String) -> Result<Manga> {
	todo!()
}

pub fn get_chapter_list(obj: Node) -> Result<Vec<Chapter>> {
	todo!()
}

pub fn parse_manga_listing(url: String) -> Result<MangaPageResult> {
	todo!()
}

pub fn get_page_list(obj: Node) -> Result<Vec<Page>> {
	todo!()
}

pub fn get_filtered_url(filters: Vec<Filter>, page: i32, url: &mut String) {
	todo!()
}

pub fn parse_incoming_url(url: String) -> String {
	todo!()
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
