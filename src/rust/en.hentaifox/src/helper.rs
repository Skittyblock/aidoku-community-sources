use aidoku::{prelude::*, std::String, std::Vec};

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
	return string;
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
		if curr < b'0' || curr > b'9' {
			break;
		}
		result = result * 10 + (curr - b'0') as i32;
	}
	if negative {
		result *= -1;
	}
	return result;
}

pub fn get_gallery_id(path: String) -> i32 {
	let parts = path.split("/").nth(2).unwrap_or("");
	return numbers_only_from_string(String::from(parts));
}

pub fn get_tag_slug(path: String) -> String {
	let parts = path.split("/").nth(2).unwrap_or("");
	return String::from(parts);
}

pub fn build_search_url(query: String, mut tags: Vec<String>, sort_type: String, page: i32) -> String {
	
	let base_url = String::from("https://hentaifox.com");
	let mut path = String::new();
	if query.len() > 0 {
		path = format!("/search?q={}&page={}", urlencode(query), page);
		if sort_type == "popular" {
			path.push_str("&sort=popular");
		}
	} else {
		if tags.len() > 0 {
			if tags[0] == "none" {
				tags.remove(0);
			}
		}
		if tags.len() > 0 {
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
	

	return url;
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
	return result;
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
		"ponytail"
	  ]
}
