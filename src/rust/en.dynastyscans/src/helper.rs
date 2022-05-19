use aidoku::{
	prelude::*, error::Result, std::String, std::ObjectRef, std::ArrayRef, std::Vec, std::net::Request, std::net::HttpMethod, std::html::Node,
	Manga, MangaStatus, MangaContentRating, MangaViewer,
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
	string
}

pub fn find_in_array(array: ArrayRef, name: String) -> Result<Vec<ObjectRef>> {
	let mut result: Vec<ObjectRef> = Vec::new();
	for value in array {
		if value.clone().as_object()?.get("type").as_string()?.read() == name {
			result.push(value.clone().as_object()?)
		}
	}
	Ok(result)
}

pub fn string_after(string: String, after: char) -> String {
	let mut result = String::new();
	let mut found = false;
	for c in string.chars() {
		if c == after {
			found = true;
			continue;
		}
		if found {
			result.push(c);
		}
	}
	result
}

pub fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("https://dynasty-scans.com/{}.json", &id);
	let json = Request::new(url.as_str(), HttpMethod::Get).json().as_object()?;

	let cover = match json.get("cover").as_string() {
		Ok(cover_url) => format!("https://dynasty-scans.com{}", cover_url.read()),
		Err(_) => String::new(),
	};

	let title = match json.get("name").as_string() {
		Ok(title) => title.read(),
		Err(_) => String::new(),
	};

	let tags = json.get("tags").as_array()?;

	let author = match find_in_array(tags.clone(), String::from("Author")) {
		Ok(authors) => {
			if authors.len() > 0 {
				match authors[0].get("name").as_string() {
					Ok(author) => author.read(),
					Err(_) => String::new(),
				}
			} else {
				String::new()
			}
		}
		Err(_) => String::new(),
	};

	let status = match find_in_array(tags.clone(), String::from("Status")) {
		Ok(statuses) => {
			if statuses.len() > 0 {
				match statuses[0].get("name").as_string() {
					Ok(status) => match status.read().as_str() {
						"Ongoing" => MangaStatus::Ongoing,
						"Completed" => MangaStatus::Completed,
						_ => MangaStatus::Unknown,
					},
					Err(_) => MangaStatus::Unknown,
				}
			} else {
				MangaStatus::Unknown
			}
		}
		Err(_) => MangaStatus::Unknown,
	};

	let mut categories: Vec<String> = Vec::new();
	if let Ok(category_objects) = find_in_array(tags.clone(), String::from("General")) {
		for category_object in category_objects {
			if let Ok(category_name) = category_object.get("name").as_string() {
				categories.push(category_name.clone().read());
			}
		}
	}

	let share_url = format!("https://dynasty-scans.com/{}", id.clone());
	let description = match json.get("description").as_string() {
		Ok(description) => Node::new_fragment(description.read().as_bytes()).text().read(),
		Err(_) => String::new(),
	};

	Ok(Manga {
		id,
		cover,
		title,
		author,
		artist: String::new(),
		description,
		url: share_url,
		categories,
		status,
		nsfw: MangaContentRating::Nsfw,
		viewer: MangaViewer::Rtl
	})
}

pub fn string_replace(string: String, replace: char, with: char) -> String {
	let mut result = String::new();
	for c in string.chars() {
		if c == replace {
			result.push(with);
		} else {
			result.push(c);
		}
	}
	result
}