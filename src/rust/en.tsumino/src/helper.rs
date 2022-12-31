use aidoku::{
	error::Result,
	std::String,
	std::Vec,
	std::{ObjectRef, ValueRef},
	Manga, MangaContentRating, MangaStatus, MangaViewer,
};

pub fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if (b'a'..=b'z').contains(&curr)
			|| (b'A'..=b'Z').contains(&curr)
			|| (b'0'..=b'9').contains(&curr)
		{
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

pub fn get_id(value: ValueRef) -> Result<String> {
	let id = value.as_int().unwrap_or(0) as i32;
	Ok(if id != 0 {
		i32_to_string(id)
	} else {
		value.as_string()?.read()
	})
}

pub fn parse_manga(manga_obj: ObjectRef) -> Result<Manga> {
	let main = manga_obj.get("entry").as_object()?;
	let id = get_id(main.get("id"))?;
	let title = main.get("title").as_string()?.read();
	let cover = main.get("thumbnailUrl").as_string()?.read();
	Ok(Manga {
		id,
		title,
		cover,
		author: String::new(),
		artist: String::new(),
		description: String::new(),
		url: String::new(),
		categories: Vec::new(),
		status: MangaStatus::Completed,
		nsfw: MangaContentRating::Nsfw,
		viewer: MangaViewer::Rtl,
	})
}
