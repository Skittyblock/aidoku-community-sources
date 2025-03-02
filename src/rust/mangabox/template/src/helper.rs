use aidoku::{
	error::Result,
	prelude::*,
	std::{ArrayRef, String, Vec},
	Filter, FilterType, MangaStatus,
};

pub fn join_string_array(array: ArrayRef, delimeter: &str) -> String {
	let mut string = String::new();
	for (i, item) in array.enumerate() {
		let node = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		if i != 0 {
			string.push_str(delimeter);
		}
		string.push_str(node.text().read().as_str());
	}
	string
}

pub fn status_from_string(string: String) -> MangaStatus {
	let string = string.trim();
	let status = string
		.rsplit_once(' ')
		.map(|(_, last)| last)
		.unwrap_or(string);
	match status {
		"Ongoing" => MangaStatus::Ongoing,
		"Completed" => MangaStatus::Completed,
		"Hiatus" => MangaStatus::Hiatus,
		"Cancelled" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	}
}

pub fn is_numeric_char(c: char) -> bool {
	c.is_ascii_digit() || c == '.'
}

// parses ".../chapter-x" where x can be e.g. "3" or "0-2" for decimals
pub fn get_chapter_number(s: &str) -> f32 {
	const PREFIX: &str = "chapter-";

	if let Some(pos) = s.find(PREFIX) {
		let number_str = &s[pos + PREFIX.len()..];

		let number_str = number_str.replace('-', ".");

		if let Ok(number) = number_str.parse::<f32>() {
			return number;
		}
	}

	0.0
}

pub fn strip_default_chapter_title(s: String) -> String {
	const PREFIX: &str = "Chapter ";

	if let Some(pos) = s[PREFIX.len()..].find(' ') {
		return s[PREFIX.len() + pos..].into();
	}

	String::default()
}

pub fn get_search_url(
	base_url: &str,
	page: i32,
	filters: Vec<Filter>,
	supports_advanced_search: bool,
	search_path: Option<&str>,
	genres: Option<&[&str]>,
) -> Result<String> {
	if supports_advanced_search {
		// isn't used anymore (?)
		let mut include: Vec<String> = Vec::new();
		let mut exclude: Vec<String> = Vec::new();
		let mut sort = String::new();
		let mut query = String::new();
		for filter in filters {
			match filter.kind {
				FilterType::Title => {
					query = filter.value.as_string()?.read();
				}
				FilterType::Genre => match filter.value.as_int().unwrap_or(-1) {
					0 => exclude.push(get_tag_id(&filter.name)),
					1 => include.push(get_tag_id(&filter.name)),
					_ => continue,
				},
				FilterType::Sort => {
					let value = match filter.value.as_object() {
						Ok(value) => value,
						Err(_) => continue,
					};
					let index = value.get("index").as_int().unwrap_or(0);
					let option = match index {
						0 => "",
						1 => "newest",
						2 => "topview",
						3 => "az",
						_ => continue,
					};
					sort = String::from(option)
				}
				_ => continue,
			}
		}

		let mut url = format!("{base_url}/advanced_search/?page={page}");
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

		Ok(url)
	} else {
		let mut title = None;
		let mut url_filter = 0;
		let mut genre = String::new();

		for filter in filters {
			match filter.kind {
				FilterType::Title => {
					title = Some(filter.value.as_string()?.read());
					break;
				}
				FilterType::Sort => {
					let index = filter.value.as_object()?.get("index").as_int().unwrap_or(0);
					url_filter += match index {
						0 => 1,
						1 => 2,
						2 => 3,
						_ => continue,
					};
				}
				FilterType::Select => match filter.name.as_str() {
					"Status" => match filter.value.as_int().unwrap_or(0) {
						1 => url_filter += 1, // completed
						2 => url_filter += 2, // ongoing
						_ => continue,
					},
					"Genre" => {
						if let Some(genres) = genres {
							let index = filter.value.as_int().unwrap_or(0) as usize;
							if index < genres.len() {
								genre = genres[index].into();
							}
						}
					}
					_ => continue,
				},
				_ => continue,
			}
		}

		Ok(if let Some(title) = title {
			format!(
				"{base_url}{}/{}?page={page}",
				search_path.unwrap_or("/search"),
				stupidencode(title)
			)
		} else {
			format!("{base_url}/genre/{genre}?filter={url_filter}&page={page}")
		})
	}
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

pub fn get_tag_id(tag: &str) -> String {
	let id = match tag {
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
	format!("{id}")
}

pub fn stupidencode(string: String) -> String {
	let mut result = String::new();
	for c in string.chars() {
		if c.is_alphanumeric() {
			result.push(c.to_ascii_lowercase());
		} else if c == ' ' {
			result.push('_');
		}
	}
	result
}
