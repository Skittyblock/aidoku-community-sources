use aidoku::{
	std::String, std::Vec, std::html::Node, std::net::Request, std::net::HttpMethod,
	Filter, FilterType,
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

pub fn get_int_manga_id(manga_id: String, base_url: String, path: String) -> String {
	let url = base_url + "/" +
			  path.as_str()+ "/" +
			  manga_id.as_str();

	let html = Request::new(url.as_str(), HttpMethod::Get).html();

	let id_html = html.select("script#wp-manga-js-extra").html().read();
	let id = &id_html[id_html.find("manga_id").unwrap()+11..id_html.find("\"};").unwrap()];

	return String::from(id);
}

pub fn get_image_url(obj: Node) -> String {
	let mut img;
	img = obj.attr("data-src").read();
	if img.len() == 0 {
		img = obj.attr("data-lazy-src").read();
	}
	if img.len() == 0 {
		img = obj.attr("src").read();
	}
	if img.len() == 0 {
		img = obj.attr("srcset").read();

	}
	// img = img.replace("-175x238", "").replace("-350x476", "").replace("-110x150", "");
	img = String::from(img.trim());
	return img;
}


pub fn get_filtered_url(filters: Vec<Filter>, page: i32, url: &mut String, search_path: String) -> bool {
	let mut is_searching = false;
	let mut query = String::new();
	let mut search_string = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					search_string.push_str(urlencode(filter_value.read().to_lowercase()).as_str());
					is_searching = true;
				}
			},
			FilterType::Check => {
				match filter.value.as_int().unwrap_or(-1) {
					0 =>  query.push_str("&status[]=on-going"),
					1 =>  query.push_str("&status[]=on-hold"),
					2 =>  query.push_str("&status[]=canceled"),
					3 =>  query.push_str("&status[]=end"),
					_ => continue,
				}
				if filter.value.as_int().unwrap_or(-1) > 0 {
					is_searching = true;
				}
			}
			FilterType::Genre => {
				query.push_str("&genre[]=");
                if let Ok(filter_id) = filter.object.get("id").as_string() {
                    query.push_str(filter_id.read().as_str());
                }
				is_searching = true;
			},
			FilterType::Select => {
				if filter.name.as_str() == "Condition" {
					match filter.value.as_int().unwrap_or(-1) {
						0 =>  query.push_str("&op="),  // OR
						1 =>  query.push_str("&op=1"), // AND
						_ => continue,
					}
					if filter.value.as_int().unwrap_or(-1) > 0 {
						is_searching = true;
					}
				}
				if filter.name.as_str() == "Adult" {
					match filter.value.as_int().unwrap_or(-1) {
						0 =>  query.push_str(""),		 // default=
						1 =>  query.push_str("&adult=0"), // None
						2 =>  query.push_str("&adult=1"), // Only
						_ => continue,
					}
					if filter.value.as_int().unwrap_or(-1) > 0 {
						is_searching = true;
					}
				}
			},
			_ => continue,
		}
	}

	if is_searching {
		url.push_str("/");
		url.push_str(&search_path);
		url.push_str("/");
		url.push_str(&i32_to_string(page));
		url.push_str("/?s=");
		url.push_str(&search_string);
		url.push_str("&post_type=wp-manga");
		url.push_str(&query);
	}
    return is_searching;
}