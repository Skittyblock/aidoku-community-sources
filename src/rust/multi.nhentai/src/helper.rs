use aidoku::{error::Result, std::ArrayRef, std::String, std::ValueRef, std::Vec};

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

pub fn get_cover_url(id: String, filetype: String) -> String {
	let mut string = String::from("https://t.nhentai.net/galleries/");
	string.push_str(&id);
	string.push_str("/cover.");
	string.push_str(&filetype);
	string
}

pub fn get_details_url(id: String) -> String {
	let mut string = String::from("https://nhentai.net/api/gallery/");
	string.push_str(&id);
	string
}

pub fn get_file_type(filetype: String) -> String {
	return match filetype.as_str() {
		"j" => String::from("jpg"),
		"p" => String::from("png"),
		"w" => String::from("webp"),
		"g" => String::from("gif"),
		_ => String::new(),
	};
}

pub fn get_tag_names_by_type(tags: ArrayRef, tag_type: &str) -> Result<Vec<String>> {
	let mut names: Vec<String> = Vec::new();
	for tag in tags {
		let tag_obj = tag.as_object()?;
		if tag_obj.get("type").as_string()?.read() == tag_type {
			names.push(tag_obj.get("name").as_string()?.read());
		}
	}
	if names.is_empty() {
		names.push(String::new());
	}
	Ok(names)
}

pub fn get_id(value: ValueRef) -> Result<String> {
	let id = value.as_int().unwrap_or(0) as i32;
	Ok(if id != 0 {
		i32_to_string(id)
	} else {
		value.as_string()?.read()
	})
}

pub fn is_number(s: &str) -> bool {
	s.chars().all(|c| c.is_numeric())
}
