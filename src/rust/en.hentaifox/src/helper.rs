use aidoku::{prelude::*, std::String, std::Vec};

pub fn urlencode(string: String) -> String {
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
		if !(b'0'..=b'9').contains(&curr) {
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
	query: String,
	mut tags: Vec<String>,
	sort_type: String,
	page: i32,
) -> String {
	let base_url = String::from("https://hentaifox.com");
	let mut path = String::new();
	if !query.is_empty() {
		path = format!("/search?q={}&page={}", urlencode(query), page);
		if sort_type == "popular" {
			path.push_str("&sort=popular");
		}
	} else {
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
			if sort_type == "popular" {
				path.push_str("/popular");
			}
			if page > 1 {
				if page > 2 {
					path.push_str(format!("/pag/{}", page).as_str());
				} else {
					path.push_str(format!("/page/{}", page).as_str());
				}
			}
		}
	}

	let url = format!("{}{}", base_url, path);

	url
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
