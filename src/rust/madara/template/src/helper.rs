use aidoku::{
	std::defaults::defaults_get, std::html::Node, std::net::HttpMethod, std::net::Request,
	std::String, std::Vec, Filter, FilterType,
};

use crate::template::MadaraSiteData;

extern crate alloc;
use alloc::string::ToString;

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

pub fn img_url_encode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if curr == b'-' {
			result.push(b'-');
		} else if curr == b'?' {
			result.push(b'?');
		} else if curr == b'%' {
			result.push(b'%');
		} else if curr == b'.' {
			result.push(b'.');
		} else if curr == b'_' {
			result.push(b'_');
		} else if curr.is_ascii_lowercase() || curr.is_ascii_uppercase() || curr.is_ascii_digit() {
			result.push(curr);
		} else {
			result.push(b'%');
			if hex[curr as usize >> 4] >= 97 && hex[curr as usize >> 4] <= 122 {
				result.push(hex[curr as usize >> 4] - 32);
			} else {
				result.push(hex[curr as usize >> 4]);
			}
			if hex[curr as usize & 15] >= 97 && hex[curr as usize & 15] <= 122 {
				result.push(hex[curr as usize & 15] - 32);
			} else {
				result.push(hex[curr as usize & 15]);
			}
		}
	}
	String::from_utf8(result).unwrap_or_default()
}

pub fn get_image_url(obj: Node) -> String {
	let mut img = obj.attr("data-src").read();
	if img.is_empty() {
		img = obj.attr("data-lazy-src").read();
	}
	if img.is_empty() {
		img = obj.attr("src").read();
	}
	if img.is_empty() {
		img = obj.attr("srcset").read();
	}
	img = String::from(img.trim());

	if let Ok(highres) = defaults_get("highres") {
		if highres.as_bool().unwrap_or(false) && !img.contains("width") {
			img = img
				.replace("-350x476", "")
				.replace("-193x278", "")
				.replace("-110x150", "")
				.replace("-175x238", "");
		}
	}
	// encoding last part of the url as some scanlations use non-alphanumerical
	// chars which need to be encoded
	let img_split = img.split('/').collect::<Vec<&str>>();
	let last_encoded = img_url_encode(String::from(img_split[img_split.len() - 1]));

	let mut encoded_img = String::new();

	for item in img_split.iter().take(img_split.len() - 1) {
		encoded_img.push_str(item);
		encoded_img.push('/');
	}
	encoded_img.push_str(&last_encoded);
	encoded_img
}

pub fn get_filtered_url(filters: Vec<Filter>, page: i32, data: &MadaraSiteData) -> (String, bool) {
	let mut is_searching = false;
	let mut query = String::new();
	let mut search_string = String::new();
	let mut url = data.base_url.clone();
	let post_type = String::from("&post_type=") + &data.post_type.clone();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					search_string.push_str(urlencode(filter_value.read().to_lowercase()).as_str());
					is_searching = true;
				}
			}
			FilterType::Author => {
				if let Ok(filter_value) = filter.value.as_string() {
					query.push_str("&author=");
					query.push_str(&urlencode(filter_value.read()));
				}
			}
			FilterType::Check => {
				if filter.value.as_int().unwrap_or(-1) <= 0 {
					continue;
				}
				if filter.name == data.status_filter_cancelled {
					query.push_str("&status[]=canceled");
				} else if filter.name == data.status_filter_completed {
					query.push_str("&status[]=end");
				} else if filter.name == data.status_filter_on_hold {
					query.push_str("&status[]=on-hold");
				} else if filter.name == data.status_filter_ongoing {
					query.push_str("&status[]=on-going");
				}

				is_searching = true;
			}
			FilterType::Genre => {
				query.push_str("&genre[]=");
				if let Ok(filter_id) = filter.object.get("id").as_string() {
					query.push_str(filter_id.read().as_str());
					is_searching = true;
				}
			}
			FilterType::Select => {
				if filter.name == data.genre_condition {
					match filter.value.as_int().unwrap_or(-1) {
						0 => query.push_str("&op="),  // OR
						1 => query.push_str("&op=1"), // AND
						_ => continue,
					}
					if filter.value.as_int().unwrap_or(-1) > 0 {
						is_searching = true;
					}
				}
				if filter.name == data.adult_string {
					match filter.value.as_int().unwrap_or(-1) {
						0 => query.push_str(""),         // default
						1 => query.push_str("&adult=0"), // None
						2 => query.push_str("&adult=1"), // Only
						_ => continue,
					}
					if filter.value.as_int().unwrap_or(-1) > 0 {
						is_searching = true;
					}
				}
			}
			_ => continue,
		}
	}

	if is_searching {
		url.push('/');
		url.push_str(&data.search_path);
		url.push('/');
		url.push_str(&page.to_string());
		url.push_str("/?s=");
		url.push_str(&search_string);
		url.push_str(&post_type);
		url.push_str(&query);
	}
	(url, is_searching)
}

pub fn add_user_agent_header(mut req: Request, user_agent: &Option<String>) -> Request {
	if let Some(agent) = user_agent {
		req = req.header("User-Agent", agent);
	}
	req
}

pub fn get_int_manga_id(
	manga_id: String,
	base_url: String,
	path: String,
	user_agent: Option<String>,
) -> String {
	let url = base_url + "/" + path.as_str() + "/" + manga_id.as_str();

	let mut req = Request::new(url.as_str(), HttpMethod::Get);
	req = add_user_agent_header(req, &user_agent);

	if let Ok(html) = req.html() {
		let id_html = html.select("script#wp-manga-js-extra").html().read();
		let id = &id_html[id_html.find("manga_id").expect("Could not find manga_id") + 11
			..id_html.find("\"}").expect("Could not find end of manga_id")];
		String::from(id)
	} else {
		String::new()
	}
}

pub fn get_lang_code() -> Option<String> {
	if let Ok(languages_val) = defaults_get("languages") {
		if let Ok(languages) = languages_val.as_array() {
			if let Ok(language) = languages.get(0).as_string() {
				return Some(language.read());
			}
		}
	}
	None
}
