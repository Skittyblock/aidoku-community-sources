use aidoku::{helpers::uri::QueryParameters, prelude::*, std::String, std::Vec};
use alloc::string::ToString;

pub const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_1_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";
pub const BASE_URL: &str = "https://hentaifox.com";

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

// numbers only from string as i32
pub fn numbers_only_from_string(string: String) -> i32 {
	let mut result: i32 = 0;
	let mut index = 0;
	let mut negative = false;
	let mut length = string.len();
	if length == 0 {
		return result;
	}
	if string.as_bytes()[0] == b'-' {
		negative = true;
		index = 1;
		length -= 1;
	}
	for i in index..length {
		let curr = string.as_bytes()[i];
		if !curr.is_ascii_digit() {
			break;
		}
		result = result * 10 + (curr - b'0') as i32;
	}
	if negative {
		result *= -1;
	}
	result
}

pub fn get_gallery_id(path: String) -> i32 {
	let parts = path.split('/').nth(2).unwrap_or("");
	numbers_only_from_string(String::from(parts))
}

pub fn get_tag_slug(path: String) -> String {
	let parts = path.split('/').nth(2).unwrap_or("");
	String::from(parts)
}

pub fn build_search_url(
	term: Option<String>,
	mut tags: Vec<String>,
	sort_type: String,
	page: i32,
) -> String {
	if term.is_some() {
		let mut query = QueryParameters::new();
		query.set("q", Some(term.unwrap_or_default().as_str()));
		query.set("page", Some(page.to_string().as_str()));
		if sort_type == "popular" {
			query.set("sort", Some("popular"));
		}

		format!("{BASE_URL}/search?{query}")
	} else {
		let mut path = String::new();
		if !tags.is_empty() && tags[0] == "none" {
			tags.remove(0);
		}
		if !tags.is_empty() {
			let tag = &tags[0];
			if sort_type == "popular" {
				path = format!("/tag/{}/popular/pag/{}", tag, page);
			} else {
				path = format!("/tag/{}/pag/{}", tag, page);
			}
		} else {
			path.push_str("/language/english");
			if sort_type == "popular" {
				path.push_str("/popular");
			}
			if page > 1 {
				path.push_str(format!("/pag/{}", page).as_str());
			}
		}
		format!("{BASE_URL}{path}")
	}
}

pub fn only_chars_from_string(str: String) -> String {
	// only remove numbers
	let mut result = String::new();
	for c in str.chars() {
		if c.is_ascii_digit() {
			continue;
		}
		result.push(c);
	}
	result
}

pub fn tag_list() -> [&'static str; 51] {
	[
		"none",
		"big-breasts",
		"sole-female",
		"sole-male",
		"nakadashi",
		"anal",
		"group",
		"stockings",
		"blowjob",
		"rape",
		"lolicon",
		"schoolgirl-uniform",
		"glasses",
		"ahegao",
		"shotacon",
		"incest",
		"full-color",
		"defloration",
		"x-ray",
		"multi-work-series",
		"bondage",
		"milf",
		"yaoi",
		"mosaic-censorship",
		"double-penetration",
		"femdom",
		"paizuri",
		"males-only",
		"impregnation",
		"mind-break",
		"sex-toys",
		"dark-skin",
		"hairy",
		"netorare",
		"big-penis",
		"cheating",
		"uncensored",
		"ffm-threesome",
		"sweating",
		"sister",
		"schoolgirl",
		"futanari",
		"yuri",
		"dilf",
		"big-ass",
		"swimsuit",
		"full-censorship",
		"collar",
		"schoolboy-uniform",
		"twintails",
		"ponytail",
	]
}
