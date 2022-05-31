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

pub struct Tag {
	pub name: String,
	pub slug: String,
}

pub fn tag_list() -> Vec<Tag> {
	let mut tags: Vec<Tag> = Vec::new();
	tags.push(Tag {
		name: String::from("None"),
		slug: String::from("none"),
	});
	tags.push(Tag {
		name: String::from("big breasts"),
		slug: String::from("big-breasts"),
	});

	tags.push(Tag {
		name: String::from("sole female"),
		slug: String::from("sole-female")
	});

	tags.push(Tag {
		name: String::from("sole male"),
		slug: String::from("sole-male")
	});

	tags.push(Tag {
		name: String::from("nakadashi"),
		slug: String::from("nakadashi"),
	});
	tags.push(Tag {
		name: String::from("anal"),
		slug: String::from("anal"),
	});
	tags.push(Tag {
		name: String::from("group"),
		slug: String::from("group"),
	});
	tags.push(Tag {
		name: String::from("stockings"),
		slug: String::from("stockings"),
	});
	tags.push(Tag {
		name: String::from("blowjob"),
		slug: String::from("blowjob"),
	});

	tags.push(Tag {
		name: String::from("rape"),
		slug: String::from("rape")
	});

	tags.push(Tag {
		name: String::from("lolicon"),
		slug: String::from("lolicon"),
	});

	tags.push(Tag {
		name: String::from("schoolgirl uniform"),
		slug: String::from("schoolgirl-uniform"),
	});

	tags.push(Tag {
		name: String::from("glasses"),
		slug: String::from("glasses"),
	});
	
	tags.push(Tag {
		name: String::from("ahegao"),
		slug: String::from("ahegao"),
	});

	tags.push(Tag {
		name: String::from("shotacon"),
		slug: String::from("shotacon"),
	});

	tags.push(Tag {
		name: String::from("incest"),
		slug: String::from("incest"),
	});

	tags.push(Tag {
		name: String::from("full color"),
		slug: String::from("full-color"),
	});

	tags.push(Tag {
		name: String::from("defloration"),
		slug: String::from("defloration"),
	});

	tags.push(Tag {
		name: String::from("x-ray"),
		slug: String::from("x-ray"),
	});

	tags.push(Tag {
		name: String::from("multi-work series"),
		slug: String::from("multi-work-series"),
	});

	tags.push(Tag {
		name: String::from("bondage"),
		slug: String::from("bondage"),
	});

	tags.push(Tag {
		name: String::from("milf"),
		slug: String::from("milf")
	});

	tags.push(Tag {
		name: String::from("yaoi"),
		slug: String::from("yaoi"),
	});

	tags.push(Tag {
		name: String::from("mosaic censorship"),
		slug: String::from("mosaic-censorship"),
	});

	tags.push(Tag {
		name: String::from("double penetration"),
		slug: String::from("double-penetration"),
	});

	tags.push(Tag {
		name: String::from("femdom"),
		slug: String::from("femdom"),
	});

	tags.push(Tag {
		name: String::from("paizuri"),
		slug: String::from("paizuri"),
	});

	tags.push(Tag {
		name: String::from("males only"),
		slug: String::from("males-only")
	});

	tags.push(Tag {
		name: String::from("impregnation"),
		slug: String::from("impregnation"),
	});

	tags.push(Tag {
		name: String::from("mind break"),
		slug: String::from("mind-break"),
	});

	tags.push(Tag {
		name: String::from("sex toys"),
		slug: String::from("sex-toys")
	});

	tags.push(Tag {
		name: String::from("dark skin"),
		slug: String::from("dark-skin"),
	});

	tags.push(Tag {
		name: String::from("hairy"),
		slug: String::from("hairy"),
	});

	tags.push(Tag {
		name: String::from("netorare"),
		slug: String::from("netorare"),
	});
	
	tags.push(Tag {
		name: String::from("big penis"),
		slug: String::from("big-penis")
	});

	tags.push(Tag {
		name: String::from("cheating"),
		slug: String::from("cheating"),
	});

	tags.push(Tag {
		name: String::from("uncensored"),
		slug: String::from("uncensored"),
	});

	tags.push(Tag {
		name: String::from("ffm threesome"),
		slug: String::from("ffm-threesome"),
	});

	tags.push(Tag {
		name: String::from("sweating"),
		slug: String::from("sweating"),
	});

	tags.push(Tag {
		name: String::from("sister"),
		slug: String::from("sister"),
	});

	tags.push(Tag {
		name: String::from("schoolgirl"),
		slug: String::from("schoolgirl"),
	});

	tags.push(Tag {
		name: String::from("futanari"),
		slug: String::from("futanari"),
	});

	tags.push(Tag {
		name: String::from("yuri"),
		slug: String::from("yuri"),
	});

	tags.push(Tag {
		name: String::from("dilf"),
		slug: String::from("dilf"),
	});

	tags.push(Tag {
		name: String::from("big ass"),
		slug: String::from("big-ass"),
	});

	tags.push(Tag {
		name: String::from("swimsuit"),
		slug: String::from("swimsuit"),
	});
	
	tags.push(Tag {
		name: String::from("full censorship"),
		slug: String::from("full-censorship"),
	});

	tags.push(Tag {
		name: String::from("collar"),
		slug: String::from("collar"),
	});

	tags.push(Tag {
		name: String::from("schoolboy uniform"),
		slug: String::from("schoolboy-uniform"),
	});


	tags.push(Tag {
		name: String::from("twintails"),
		slug: String::from("twintails"),
	});

	tags.push(Tag {
		name: String::from("ponytail"),
		slug: String::from("ponytail"),
	});


	

	
	tags

	
}
