use crate::helper::get_id;
use aidoku::{
	error::Result,
	prelude::*,
	std::ObjectRef,
	std::String,
	std::{html::Node, Vec},
	Manga, MangaContentRating, MangaStatus, MangaViewer,
};

pub fn parse_list(manga_obj: ObjectRef) -> Result<Manga> {
	let main = manga_obj.get("entry").as_object()?;
	let id = get_id(main.get("id"))?;
	let title = main.get("title").as_string()?.read();
	let cover = match main.get("thumbnailUrl").as_string() {
		Ok(cover) => cover.read(),
		Err(_) => String::new(),
	};
	Ok(Manga {
		id,
		title,
		cover,
		status: MangaStatus::Completed,
		nsfw: MangaContentRating::Nsfw,
		viewer: MangaViewer::Rtl,
		..Default::default()
	})
}

pub fn parse_manga(html: Node) -> Result<Manga> {
	let title = html
		.select("meta[property=og:title]")
		.attr("content")
		.read();
	let author = html
		.select("div.book-page-container #Artist a")
		.array()
		.map(|val| val.as_node().expect("Failed to get author").text().read())
		.collect::<Vec<String>>()
		.join(", ");
	let thumbnail = html.select("img").attr("src").read();
	let description = get_description(html.select("div.book-info-container"));
	let tags = html
		.select("div.book-info-container #Tag a")
		.array()
		.map(|val| val.as_node().expect("Failed to get tags").text().read())
		.collect::<Vec<String>>();
	Ok(Manga {
		title,
		cover: thumbnail,
		author,
		description,
		categories: tags,
		status: MangaStatus::Completed,
		nsfw: MangaContentRating::Nsfw,
		viewer: MangaViewer::Rtl,
		..Default::default()
	})
}

fn get_description(info_element: Node) -> String {
	let mut description = String::new();
	let pages = info_element.select("#Pages").text().read();
	let parodies = info_element.select("#Parody a").array();
	let characters = info_element.select("#Characters a").array();
	description.push_str(format!("Pages: {}", pages).as_str());
	if !parodies.is_empty() {
		description.push_str("\n\nParodies: ");
		let p: Vec<String> = parodies
			.map(|val| val.as_node().expect("Failed to get parodies").text().read())
			.collect::<Vec<String>>();
		description.push_str(p.join(", ").as_str());
	}
	if !characters.is_empty() {
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
