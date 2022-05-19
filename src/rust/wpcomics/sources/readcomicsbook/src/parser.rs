use aidoku::{
	error::Result,
	prelude::format,
	std::{ObjectRef, String, Vec},
	Manga, MangaContentRating, MangaStatus, MangaViewer,
};

pub fn parse_comic(base_url: String, comic_object: ObjectRef) -> Result<Manga> {
	let slug = comic_object.get("slug").as_string()?.read();
	let title = comic_object.get("title").as_string()?.read();
	let mut img_url = String::new();
	if let Ok(url) = comic_object.get("img_url").as_string() {
		img_url = url.read();
	}
	Ok(Manga {
		id: format!("{base_url}/comic/{slug}"),
		cover: img_url,
		title,
		author: String::new(),
		artist: String::new(),
		description: String::new(),
		url: format!("{base_url}/comic/{slug}"),
		categories: Vec::new(),
		status: MangaStatus::Unknown,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Ltr,
	})
}
