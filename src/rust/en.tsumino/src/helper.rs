use aidoku::{
	error::Result,
	prelude::*,
	std::String,
	std::{html::Node, Vec},
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

pub fn parse_list(manga_obj: ObjectRef) -> Result<Manga> {
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

pub fn parse_manga(html: Node) -> Result<Manga> {
	let title = html
		.select("meta[property=og:title]")
		.attr("content")
		.read();
	let author = html
		.select("div.book-page-container")
		.select("#Artist")
		.select("a")
		.array()
		.map(|val| val.as_node().expect("Failed to get author").text().read())
		.collect::<Vec<String>>()
		.join(", ");
	let thumbnail = html.select("img").attr("src").read();
	let description = get_description(html.select("div.book-info-container"));
	let tags = html
		.select("div.book-info-container")
		.select("#Tag")
		.select("a")
		.array()
		.map(|val| val.as_node().expect("Failed to get tags").text().read())
		.collect::<Vec<String>>();
	Ok(Manga {
		id: String::new(),
		title,
		cover: thumbnail,
		author,
		artist: String::new(),
		description,
		url: String::new(),
		categories: tags,
		status: MangaStatus::Completed,
		nsfw: MangaContentRating::Nsfw,
		viewer: MangaViewer::Rtl,
	})
}

fn get_description(info_element: Node) -> String {
	let mut description = String::new();
	let pages = info_element.select("#Pages").text().read();
	let parodies = info_element.select("#Parody").select("a").array();
	let characters = info_element.select("#Characters").select("a").array();
	description.push_str(format!("Pages: {}", pages).as_str());
	if parodies.len() > 0 {
		description.push_str("\n\nParodies: ");
		let p: Vec<String> = parodies
			.map(|val| val.as_node().expect("Failed to get parodies").text().read())
			.collect::<Vec<String>>();
		description.push_str(p.join(", ").as_str());
	}
	if characters.len() > 0 {
		description.push_str("\n\nCharacters: ");
		let characters: Vec<String> = characters
			.map(|val| {
				val.as_node()
					.expect("Failed to get characters")
					.text()
					.read()
			})
			.collect::<Vec<String>>();
		description.push_str(characters.join(", ").as_str());
	}
	description
}
