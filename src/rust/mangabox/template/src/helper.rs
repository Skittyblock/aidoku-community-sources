use aidoku::{std::ArrayRef, std::String, std::Vec, MangaStatus};

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

pub fn join_string_array(array: ArrayRef, delimeter: String) -> String {
	let mut string = String::new();
	for (i, item) in array.enumerate() {
		let node = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		if i != 0 {
			string.push_str(&delimeter);
		}
		string.push_str(node.text().read().as_str());
	}
	string
}

pub fn status_from_string(status: String) -> MangaStatus {
	if status == "Ongoing" {
		MangaStatus::Ongoing
	} else if status == "Completed" {
		MangaStatus::Completed
	} else if status == "Hiatus" {
		MangaStatus::Hiatus
	} else if status == "Cancelled" {
		MangaStatus::Cancelled
	} else {
		MangaStatus::Unknown
	}
}

pub fn is_numeric_char(c: char) -> bool {
	c.is_ascii_digit() || c == '.'
}

pub fn get_chapter_number(id: String) -> f32 {
	let mut number_string = String::new();
	let mut i = id.len() - 1;
	for c in id.chars().rev() {
		if !is_numeric_char(c) {
			number_string = String::from(&id[i + 1..]);
			break;
		}
		i -= 1;
	}
	if number_string.is_empty() {
		return 0.0;
	}
	number_string.parse::<f32>().unwrap_or(0.0)
}

pub fn get_search_url(
	base_url: String,
	query: String,
	page: i32,
	include: Vec<String>,
	exclude: Vec<String>,
	sort: String,
) -> String {
	let mut url = String::new();
	url.push_str(&base_url);
	url.push_str("/advanced_search/?page=");
	url.push_str(&i32_to_string(page));
	if !query.is_empty() {
		url.push_str("&keyw=");
		url.push_str(&stupidencode(query));
	}
	if !include.is_empty() {
		url.push_str("&g_i=");
		for (i, tag) in include.iter().enumerate() {
			if i == 0 {
				url.push('_');
			}
			url.push_str(tag.as_str());
			url.push('_');
		}
	}
	if !exclude.is_empty() {
		url.push_str("&g_e=");
		for (i, tag) in exclude.iter().enumerate() {
			if i == 0 {
				url.push('_');
			}
			url.push_str(tag.as_str());
			url.push('_');
		}
	}
	if !sort.is_empty() {
		url.push_str("&orby=");
		url.push_str(sort.as_str());
	}
	url
}

pub fn string_replace(string: String, search: String, replace: String) -> String {
	let mut result = String::new();
	let mut at = 0;
	for c in string.chars() {
		if c == search.chars().next().unwrap() {
			if string[at..].starts_with(&search) {
				result.push_str(&replace);
				at += search.len();
			} else {
				result.push(c);
			}
		} else {
			result.push(c);
		}
		at += 1;
	}
	result
}

pub fn get_tag_id(tag: String) -> String {
	let id = match tag.as_str() {
		"Action" => 2,
		"Adult" => 3,
		"Adventure" => 4,
		"Comedy" => 6,
		"Cooking" => 7,
		"Doujinshi" => 9,
		"Drama" => 10,
		"Ecchi" => 11,
		"Fantasy" => 12,
		"Gender bender" => 13,
		"Harem" => 14,
		"Historical" => 15,
		"Horror" => 16,
		"Isekai" => 45,
		"Josei" => 17,
		"Manhua" => 44,
		"Manhwa" => 43,
		"Martial arts" => 19,
		"Mature" => 20,
		"Mecha" => 21,
		"Medical" => 22,
		"Mystery" => 24,
		"One shot" => 25,
		"Psychological" => 26,
		"Romance" => 27,
		"School life" => 28,
		"Sci fi" => 29,
		"Seinen" => 30,
		"Shoujo" => 31,
		"Shoujo ai" => 32,
		"Shounen" => 33,
		"Shounen ai" => 34,
		"Slice of life" => 35,
		"Smut" => 36,
		"Sports" => 37,
		"Supernatural" => 38,
		"Tragedy" => 39,
		"Webtoons" => 40,
		"Yaoi" => 41,
		"Yuri" => 42,
		_ => -1,
	};
	i32_to_string(id)
}

pub fn stupidencode(string: String) -> String {
	let mut result = String::new();
	for c in string.chars() {
		if c.is_alphanumeric() {
			result.push(c);
		} else if c == ' ' {
			result.push('_');
		}
	}
	result
}
