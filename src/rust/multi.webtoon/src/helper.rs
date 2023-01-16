use aidoku::{
	prelude::format,
	std::defaults::defaults_get,
	std::net::{HttpMethod, Request},
	std::{String, Vec},
	Filter, FilterType,
};

/// Returns the correct url for the selected language and mobile/desktop
pub fn get_base_url(mobile: bool) -> String {
	let desktop_base_url = "https://www.webtoons.com";
	let mobile_base_url = "https://m.webtoons.com";
	let lang = get_lang_code().unwrap_or(String::from("en"));

	if mobile {
		format!("{}/{}", mobile_base_url, lang)
	} else {
		format!("{}/{}", desktop_base_url, lang)
	}
}

/// Returns the currently selected language
pub fn get_lang_code() -> Option<String> {
	if let Ok(languages) = defaults_get("languages")
		.expect("Could not read language")
		.as_array()
	{
		if let Ok(language) = languages.get(0).as_string() {
			return Some(language.read());
		}
	}
	None
}

/// Returns a useragent string
pub fn get_user_agent(mobile: bool) -> String {
	if mobile {
		String::from("Mozilla/5.0 (iPhone; CPU iPhone OS 16_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.1 Mobile/15E148 Safari/604.1")
	} else {
		String::from("Mozilla/5.0 (Macintosh; Intel Mac OS X 13_1) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.1 Safari/605.1.15")
	}
}

/// Request wrapper to set cookies for each request
pub fn request(url: &String, mobile: bool) -> Request {
	let locale = get_lang_code().unwrap_or(String::from("en"));
	let age_gate_pass = true;
	let need_gdpr = true;
	let need_ccpa = true;
	let need_coppa = false;
	let user_agent = get_user_agent(mobile);

	let cookie_string = format!(
		"locale={}; ageGatePass={}; needGDPR={}; needCCPA={}; needCOPPA={}",
		locale, age_gate_pass, need_gdpr, need_ccpa, need_coppa
	);

	Request::new(url, HttpMethod::Get)
		.header("Referer", &get_base_url(false))
		.header("Cookie", &cookie_string)
		.header("User-Agent", &user_agent)
}

/// Percent-encode any non-ASCII characters in a string.
pub fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789ABCDEF".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if curr.is_ascii_alphanumeric() {
			result.push(curr);
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}
	String::from_utf8(result).unwrap_or_default()
}

/// Returns the ID of a manga from a URL.
pub fn get_manga_id(url: String) -> String {
	// Example Url: https://www.webtoons.com/en/action/ultra-alternate-character/list?title_no=3581
	// Example Url: https://www.webtoons.com/episodeList?titleNo=3581
	// parse "3581" from the url

	// Webtoons also has a different category called "Canvas" titles which are in a different format.
	// They contain "challenge" in the url, so we have to account for that
	// Simple solution, append "challenge" to the id if it's a canvas title
	// Example Url: https://www.webtoons.com/en/challenge/meme-girls/list?title_no=304446
	// Example Url: https://www.webtoons.com/challenge/episodeList?titleNo=304446
	// parse "304446-challenge" from the url

	if url.contains("title_no=") || url.contains("titleNo=") {
		let split_url = {
			if url.contains("title_no=") {
				url.split("title_no=").collect::<Vec<&str>>()
			} else if url.contains("titleNo=") {
				url.split("titleNo=").collect::<Vec<&str>>()
			} else {
				Vec::new()
			}
		};

		if !split_url.is_empty() {
			let manga_id = split_url[1];

			// Append "challenge" to the id if it's a canvas title
			if url.contains("challenge") {
				format!("{}-challenge", manga_id)
			} else {
			String::from(manga_id)
			}
		} else {
			String::new()
		}
	} else {
		String::new()
	}
}

/// Returns the ID of a chapter from a URL.
pub fn get_chapter_id(url: String) -> String {
	// Example Url: https://www.webtoons.com/en/action/ultra-alternate-character/ep-1-healer-servant/viewer?title_no=3581&episode_no=1
	// Example Url: https://www.webtoons.com/viewer?titleNo=3581&episodeNo=1
	// parse "1" from the url

	if url.contains("episode_no") || url.contains("episodeNo") {
		let split_url = {
			if url.contains("episode_no") {
				url.split("episode_no=").collect::<Vec<&str>>()
			} else if url.contains("episodeNo") {
				url.split("episodeNo=").collect::<Vec<&str>>()
			} else {
				Vec::new()
			}
		};

		if !split_url.is_empty() {
			let chapter_id = split_url[1];
			String::from(chapter_id)
		} else {
			String::new()
		}
	} else {
		String::new()
	}
}

/// Returns full URL of a manga from a manga ID.
pub fn get_manga_url(manga_id: String, base_url: String) -> String {
	// Example manga id: 3581
	// return "https://www.webtoons.com/episodeList?titleNo=3581"

	// Removing the language tag from the url, because it is not required
	let mut split_url = base_url.split('/').collect::<Vec<&str>>();
	split_url.pop();

	let base_url = split_url.join("/");

	format!("{}/episodeList?titleNo={}", base_url, manga_id)
}

/// Returns full URL of a chapter from a chapter ID and manga ID.
pub fn get_chapter_url(chapter_id: String, manga_id: String, base_url: String) -> String {
	// Example chapter id: 1
	// Example manga id: 3581
	// return "https://www.webtoons.com/viewer?titleNo=3581&episodeNo=1"

	// Removing the language tag from the url, because it is not required
	let mut split_url = base_url.split('/').collect::<Vec<&str>>();
	split_url.pop();

	let base_url = split_url.join("/");

	format!(
		"{}/viewer?titleNo={}&episodeNo={}",
		base_url, manga_id, chapter_id
	)
}

/// Returns the search status as a boolean and the search string if there is one
pub fn check_for_search(filters: Vec<Filter>) -> (String, bool) {
	let mut search_string = String::new();
	let mut search = false;

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					search_string.push_str(urlencode(filter_value.read().to_lowercase()).as_str());
					search = true;
					break;
				}
			}
			_ => continue,
		}
	}
	(search_string, search)
}
