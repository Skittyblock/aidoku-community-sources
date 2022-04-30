use aidoku::{
	std::String, std::ArrayRef, std::Vec, MangaStatus,
};

pub fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();
	
	for byte in bytes {
		let curr = *byte;
		if (b'a' <= curr && curr <= b'z')
			|| (b'A' <= curr && curr <= b'Z')
			|| (b'0' <= curr && curr <= b'9') {
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

pub fn join_string_array(array: ArrayRef, delimeter: String) -> String {
	let mut string = String::new();
	let mut at = 0;
	for item in array {
		if at != 0 {
			string.push_str(&delimeter);
		}
		string.push_str(item.as_node().text().read().as_str());
		at += 1;
	}
	return string;
}

pub fn status_from_string(status: String) -> MangaStatus {
	if status == "Ongoing" {
		return MangaStatus::Ongoing;
	} else if status == "Completed" {
		return MangaStatus::Completed;
	} else if status == "Hiatus" {
		return MangaStatus::Hiatus;
	} else if status == "Cancelled" {
		return MangaStatus::Cancelled;
	} else {
		return MangaStatus::Unknown;
	}
}

pub fn is_numeric_char(c: char) -> bool {
	return (c >= '0' && c <= '9') || c == '.';
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
	if number_string.len() == 0 {
		return 0.0;
	}
	return number_string.parse::<f32>().unwrap_or(0.0);
}

pub fn get_search_url(base_url: String, query: String, page: i32, include: Vec<String>, exclude: Vec<String>, sort: String) -> String {
	let mut url = String::new();
	url.push_str(&base_url);
	url.push_str("/advanced_search/?page=");
	url.push_str(&i32_to_string(page));
	if query.len() > 0 {
		url.push_str("&keyw=");
		url.push_str(&stupidencode(query));
	}
	if include.len() > 0 {
		url.push_str("&g_i=");
		for (i, tag) in include.iter().enumerate() {
			if i == 0 {
				url.push_str("_");
			}
			url.push_str(tag.as_str());
			url.push_str("_");
		}
	}
	if exclude.len() > 0 {
		url.push_str("&g_e=");
		for (i, tag) in exclude.iter().enumerate() {
			if i == 0 {
				url.push_str("_");
			}
			url.push_str(tag.as_str());
			url.push_str("_");
		}
	}
	if sort.len() > 0 {
		url.push_str("&orby=");
		url.push_str(sort.as_str());
	}
	return url;
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
	return result;
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
	return i32_to_string(id);
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
	return result;
}