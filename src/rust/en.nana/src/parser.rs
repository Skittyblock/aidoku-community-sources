use aidoku::{
	prelude::*, error::Result, std::{String, ObjectRef}, std::Vec, std::html::Node,
	Filter, FilterType, Manga, Page, MangaStatus, MangaContentRating, MangaViewer
};

const BASE_URL: &str = "https://nana.my.id";

pub fn parse_search(html: Node, result: &mut Vec<Manga>) {
	for page in html.select("#thumbs_container > .id1").array() {
		let obj = page.as_node();
		let uwobj = obj.expect("Node Object");
		let a = uwobj.select(".id3 > a");
		let id: String = a.attr("href")
		.read()
		.split("/")
		.last()
		.unwrap()
		.into();
		
		let url = format!("{}/reader/{}", BASE_URL, &id);
		let title = a.attr("title").read();
		let author = a.select("img").attr("alt").read().replace(&format!("{} by ", title), "");

		let img = a.select("img").attr("src").read();
		let img_url = if img.starts_with("/") {
			format!("{}{}", BASE_URL, img)
		} else {
			img
		};

		let mut categories: Vec<String> = Vec::new();
		uwobj.select(".id4 > .tags > span")
		.array()
		.for_each(|tag| categories.push(tag.as_node().unwrap().text().read()));



		if id.len() > 0 && title.len() > 0 && img_url.len() > 0 {
			result.push(Manga {
				id,
				cover: img_url,
				title,
				author: author,
				artist: String::new(),
				description: String::new(),
				url: url,
				categories: categories,
				status: MangaStatus::Completed,
				nsfw: MangaContentRating::Nsfw,
				viewer: MangaViewer::Scroll
			});
		}
	}
}

pub fn get_page_list(obj: ObjectRef) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	let mut i = 0;
	for page in obj.get("pages").as_array()? {
		let cleanid: String = page.as_string()?.read().replace("thumbnails", "pages").chars().skip(1).collect();
		let url = format!("{}{}", BASE_URL, cleanid);

		pages.push(Page {
			index: i as i32,
			url,
			base64: String::new(),
			text: String::new(),
		});
		i += 1;
	}

	Ok(pages)
}

pub fn get_filtered_url(filters: Vec<Filter>, page: i32, url: &mut String) {
	let mut is_searching = false;
	let mut search_string = String::new();
	url.push_str(BASE_URL);

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					search_string.push_str(urlencode(filter_value.read().to_lowercase()).as_str());
					is_searching = true;
				}
			}
			_ => continue,
		}
	}


	if is_searching {
		url.push_str("/?q=");
		url.push_str(&search_string);
		url.push_str("&p=");
		url.push_str(&i32_to_string(page));
	} else {
		url.push_str("?p=");
		url.push_str(&i32_to_string(page));
	}
}

// HELPER FUNCTIONS

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