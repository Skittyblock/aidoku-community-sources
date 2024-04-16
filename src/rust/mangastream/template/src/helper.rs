use core::ptr;
use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	helpers::substring::Substring,
	prelude::format,
	std::{current_date, html::Node},
	std::{defaults::defaults_get, net::Request},
	std::{String, StringRef, Vec},
	MangaStatus,
};

use crate::template::MangaStreamSource;

extern crate hashbrown;
use hashbrown::HashMap;

pub const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_1_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";

// generate url for listing page
pub fn get_listing_url(
	listing: [&str; 3],
	base_url: String,
	pathname: String,
	listing_name: String,
	page: i32,
) -> String {
	let list_type = if listing_name == listing[0] {
		"order=update"
	} else if listing_name == listing[1] {
		"order=popular"
	} else if listing_name == listing[2] {
		"order=latest"
	} else {
		""
	};
	match page {
		1 => format!("{}/{}/?{}", base_url, pathname, list_type),
		_ => format!("{}/{}/?page={}&{}", base_url, pathname, page, list_type),
	}
}

// return the manga status
pub fn manga_status(
	status: String,
	status_options: [&'static str; 5],
	status_options_2: [&'static str; 5],
) -> MangaStatus {
	if (!status_options[0].is_empty() && status.contains(status_options[0]))
		|| (!status_options_2[0].is_empty() && status.contains(status_options_2[0]))
	{
		MangaStatus::Ongoing
	} else if (!status_options[1].is_empty() && status.contains(status_options[1]))
		|| (!status_options_2[1].is_empty() && status.contains(status_options_2[1]))
	{
		MangaStatus::Completed
	} else if (!status_options[2].is_empty() && status.contains(status_options[2]))
		|| (!status_options_2[2].is_empty() && status.contains(status_options_2[2]))
	{
		MangaStatus::Hiatus
	} else if (!status_options[3].is_empty() && status.contains(status_options[3]))
		|| (!status_options[4].is_empty() && status.contains(status_options[4]))
		|| (!status_options_2[3].is_empty() && status.contains(status_options_2[3]))
		|| (!status_options_2[4].is_empty() && status.contains(status_options_2[4]))
	{
		MangaStatus::Cancelled
	} else {
		MangaStatus::Unknown
	}
}

//converts integer(i32) to string
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

/// Converts `<br>` and `\n` into newlines.
pub fn text_with_newlines(node: Node) -> String {
	let html = node.html().read();
	if !String::from(html.trim()).is_empty() {
		String::from(
			Node::new_fragment(
				node.html()
				.read()
				// This also replaces `\n` because mangastream sources split their
				// description text into multiple p tags, and this causes newlines
				// to be lost if you call `text()` on the node.
				// So to fix that we replace all newlines with a placeholder, and
				// then replace the placeholder with a newline after calling `text()`.
				.replace('\n', "{{ .LINEBREAK }}")
				.replace("<br>", "{{ .LINEBREAK }}")
				.as_bytes(),
			)
			.expect("Failed to create new fragment")
			.text()
			.read()
			.replace("{{ .LINEBREAK }}", "\n")
			.trim(),
		)
	} else {
		String::new()
	}
}

// return chapter number from string
pub fn get_chapter_number(id: String) -> f32 {
	id.chars()
		.filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.')
		.collect::<String>()
		.split(' ')
		.collect::<Vec<&str>>()
		.into_iter()
		.map(|a| a.parse::<f32>().unwrap_or(0.0))
		.find(|a| *a > 0.0)
		.unwrap_or(0.0)
}

// generates the search, filter and homepage url
pub fn get_search_url(
	source: &MangaStreamSource,
	query: String,
	page: i32,
	included_tags: Vec<String>,
	excluded_tags: Vec<String>,
	status: String,
	manga_type: String,
) -> String {
	let mut url = format!("{}/{}", source.base_url, source.traverse_pathname);
	if query.is_empty() && included_tags.is_empty() && status.is_empty() && manga_type.is_empty() {
		return get_listing_url(
			source.listing,
			source.base_url.clone(),
			String::from(source.traverse_pathname),
			String::from(source.listing[0]),
			page,
		);
	}
	if !query.is_empty() {
		url.push_str(&format!("/page/{}?s={}", page, query.replace(' ', "+")))
	} else {
		url.push_str(&format!("/?page={}", page));
	}
	if !included_tags.is_empty() || !excluded_tags.is_empty() {
		if excluded_tags.is_empty() {
			for tag in included_tags {
				url.push_str(&format!("&genre%5B%5D={}", tag));
			}
		} else if !included_tags.is_empty() && !excluded_tags.is_empty() {
			for tag in included_tags {
				url.push_str(&format!("&genre%5B%5D={}", tag));
			}
			for tag in excluded_tags {
				url.push_str(&format!("&genre%5B%5D=-{}", tag));
			}
		} else {
			for tag in excluded_tags {
				url.push_str(&format!("&genre%5B%5D=-{}", tag));
			}
		}
	}
	if !status.is_empty() {
		url.push_str(&format!("&status={}", status));
	}
	if !manga_type.is_empty() {
		url.push_str(&format!("&type={}", manga_type));
	}
	url
}

// return the date depending on the language
pub fn get_date(source: &MangaStreamSource, raw_date: StringRef) -> f64 {
	match source.base_url.contains(source.date_string) {
		true => raw_date
			.0
			.as_date(source.chapter_date_format_2, Some(source.locale_2), None)
			.unwrap_or(0.0),
		_ => raw_date
			.0
			.as_date(source.chapter_date_format, Some(source.locale), None)
			.unwrap_or(0.0),
	}
}

// encoding non alpha-numeric characters to utf8
pub fn img_url_encode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if curr == b'-' {
			result.push(b'-');
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

//get the image sources as some images are in base64 format
pub fn get_image_src(node: Node) -> String {
	let mut image = String::new();
	let src = node.select("img").first().attr("src").read();
	let data_lazy = node.select("img").first().attr("data-lazy-src").read();
	let data_src = node.select("img").first().attr("data-src").read();
	if !src.starts_with("data") && !src.is_empty() {
		image = node
			.select("img")
			.first()
			.attr("src")
			.read()
			.replace("?resize=165,225", "");
	} else if !data_lazy.starts_with("data") && !data_lazy.is_empty() {
		image = node
			.select("img")
			.first()
			.attr("data-lazy-src")
			.read()
			.replace("?resize=165,225", "");
	} else if !data_src.starts_with("data") && !data_src.is_empty() {
		image = node
			.select("img")
			.first()
			.attr("data-src")
			.read()
			.replace("?resize=165,225", "");
	}
	let img_split = image.split('/').collect::<Vec<&str>>();
	let last_encoded = img_url_encode(String::from(img_split[img_split.len() - 1]));
	let mut encoded_img = String::new();

	(0..img_split.len() - 1).for_each(|i| {
		encoded_img.push_str(img_split[i]);
		encoded_img.push('/');
	});
	encoded_img.push_str(&last_encoded);
	append_protocol(encoded_img)
}

pub fn append_protocol(url: String) -> String {
	if url.starts_with("https") || url.starts_with("http") {
		url
	} else {
		format!("{}{}", "https:", url)
	}
}

pub fn urlencode<T: AsRef<[u8]>>(url: T) -> String {
	let bytes = url.as_ref();
	let hex = "0123456789ABCDEF".as_bytes();

	let mut result: Vec<u8> = Vec::with_capacity(bytes.len() * 3);

	for byte in bytes {
		let curr = *byte;
		if curr.is_ascii_alphanumeric() || b";,/?:@&=+$-_.!~*'()#".contains(&curr) {
			result.push(curr);
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}
	String::from_utf8(result).unwrap_or_default()
}

/// This function is used to get the permanent url of a manga or chapter
///
/// This is done by removing the random number near the end of the url
///
/// This will work for most if not all sources that use randomized url's for the
/// `manga url`, but for the `chapter url` it will only work for some sources
pub fn get_permanet_url(original_url: String) -> String {
	let mut original_url = original_url;

	// remove trailing slash
	if original_url.ends_with('/') {
		original_url.pop();
	};

	// get the leading garbage from end of url
	// example https://luminousscans.com/series/1671729411-a-bad-person/
	// will return 1671729411, this random number is completely useless and
	// only exists to stop scrapers
	let garbage = original_url
		.split('/')
		.last()
		.expect("Failed to split url by /")
		.split('-')
		.next()
		.expect("Failed to split url by -");

	// check if the garbage is a 10 digit number to prevent removing the wrong part
	// of the url the garbage should always be a 10 digit number
	if garbage.parse::<u64>().is_ok() && garbage.len() == 10 {
		// remove the garbage from the url
		// example https://luminousscans.com/series/1671729411-a-bad-person/
		// will return https://luminousscans.com/series/a-bad-person
		original_url.replace(&format!("{}{}", garbage, "-"), "")
	} else {
		original_url
	}
}

/// This function is used to get the id from a url
///
/// The id is the last part of the url
pub fn get_id_from_url(url: String) -> String {
	let mut url = url;

	// remove trailing slash
	if url.ends_with('/') {
		url.pop();
	};

	// if there is a post id in the url, return it
	if url.contains("p=") {
		return String::from(
			url.substring_after("p=")
				.expect("Failed to parse id from url")
				.substring_before("&")
				.expect("Failed to parse id from url"),
		);
	}

	// this will get the last part of the url
	// example https://flamescans.org/series/the-world-after-the-fall
	// will return the-world-after-the-fall
	// example https://flamescans.org/the-world-after-the-fall-chapter-55
	// will return the-world-after-the-fall-chapter-55
	let id = url.split('/').last().expect("Failed to parse id from url");

	String::from(id)
}

pub fn get_lang_code() -> String {
	if let Ok(languages) = defaults_get("languages") {
		if let Ok(arr) = languages.as_array() {
			if let Ok(language) = arr.get(0).as_string() {
				return language.read();
			}
		}
	}
	String::new()
}

static mut CACHED_MANGA_URL_TO_POSTID_MAPPING: Option<HashMap<String, String>> = None;
static mut CACHED_MAPPING_AT: f64 = 0.0;

// This requests the "all manga" listing page in text mode and parses out
// the postid and url for each manga, and caches it in a hashmap to prevent
// having to request the page again.
//
// The all manga listing page is the only reliable way to get the postids for
// each manga, without making a request to each and every manga page when
// browsing (*cough* paperback *cough*)
//
/// Generate a hashmap of manga url to postid mappings
fn generate_manga_url_to_postid_mapping(
	url: &str,
	pathname: &str,
) -> Result<HashMap<String, String>> {
	unsafe {
		// if the mapping was generated less than 10 minutes ago, use the cached mapping
		if current_date() - CACHED_MAPPING_AT < 600.0 {
			if let Some(mapping) = &mut *ptr::addr_of_mut!(CACHED_MANGA_URL_TO_POSTID_MAPPING) {
				return Ok(mapping.clone());
			}
		}
	}

	let all_manga_listing_url = format!("{}/{}/list-mode", url, pathname);

	let html = Request::get(all_manga_listing_url)
		.header("User-Agent", USER_AGENT)
		.html()?;
	let mut mapping = HashMap::new();

	for node in html.select(".soralist .series").array() {
		let manga = node.as_node()?;

		let url = manga.attr("href").read();
		let post_id = manga.attr("rel").read();

		mapping.insert(url, post_id);
	}

	unsafe {
		CACHED_MANGA_URL_TO_POSTID_MAPPING = Some(mapping.clone());
		CACHED_MAPPING_AT = current_date();
	}

	Ok(mapping)
}

/// Search the `MANGA_URL_TO_POSTID_MAPPING` for the postid from a manga url
pub fn get_postid_from_manga_url(url: String, base_url: &str, pathname: &str) -> Result<String> {
	let manga_url_to_postid_mapping = generate_manga_url_to_postid_mapping(base_url, pathname)?;
	let id = manga_url_to_postid_mapping.get(&url).ok_or(AidokuError {
		reason: AidokuErrorKind::Unimplemented, // no better error type available
	})?;

	Ok(String::from(id))
}

// This requests the chapters via the admin ajax endpoint using post ids and
// parses out the postid and url for each chapter, and returns it in a hashmap
//
/// Generate a hashmap of chapter url to postid mappings
pub fn generate_chapter_url_to_postid_mapping(
	post_id: String,
	base_url: &str,
) -> Result<HashMap<String, String>> {
	let ajax_url = format!("{}/wp-admin/admin-ajax.php", base_url);

	let start = current_date();

	let body = format!("action=get_chapters&id={}", post_id);
	let html = Request::post(ajax_url)
		.body(body.as_bytes())
		.header("Referer", base_url)
		.header("User-Agent", USER_AGENT)
		.html()?;

	// Janky retry logic to bypass rate limiting
	// Retry after 10 seconds if we get rate limited. 10 seconds is the shortest
	// interval that we can retry without getting rate limited again.
	if html.select("title").text().read() == "429 Too Many Requests" {
		loop {
			if start + 10.0 < current_date() {
				return generate_chapter_url_to_postid_mapping(post_id, base_url);
			}
		}
	}

	let mut mapping = HashMap::new();

	for node in html.select("option").array() {
		let chapter = node.as_node()?;

		let url = chapter.attr("value").read();
		let post_id = chapter.attr("data-id").read();

		mapping.insert(url, post_id);
	}

	Ok(mapping)
}
