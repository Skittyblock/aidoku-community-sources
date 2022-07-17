use aidoku::{
	error::Result,
	std::{current_date, defaults::defaults_get, ObjectRef, String, StringRef, Vec},
	Chapter, Manga, MangaContentRating, MangaStatus, MangaViewer,
};

fn get_md_localized_string(obj: ObjectRef) -> String {
	let language = if let Ok(langs) = defaults_get("languages").as_array()
		&& let Ok(lang) = langs.get(0).as_string()
		&& defaults_get("usePreferredLanguage").as_bool().unwrap_or(false)
	{
		lang.read()
	} else {
		String::from("en")
	};

	// Try for preferred language first
	match obj.get(&language).as_string() {
		Ok(value) => value.read(),
		// Fallback to English
		Err(_) => match obj.get("en").as_string() {
			Ok(value) => value.read(),
			// Fallback to Japanese (might be romaji)
			Err(_) => match obj.get("ja-ro").as_string() {
				Ok(value) => value.read(),
				// Fuck it, get first value
				Err(_) => match obj.values().get(0).as_string() {
					Ok(value) => value.read(),
					Err(_) => String::from(""),
				},
			},
		},
	}
}

// Parse manga with title and cover
pub fn parse_basic_manga(manga_object: ObjectRef) -> Result<Manga> {
	let attributes = manga_object.get("attributes").as_object()?;
	let id = manga_object.get("id").as_string()?.read();

	// Title
	let titles = attributes.get("title").as_object()?;
	let title = get_md_localized_string(titles);

	// Cover
	let mut cover_file: String = String::new();

	if let Ok(relationships) = manga_object.get("relationships").as_array() {
		for relationship in relationships {
			if let Ok(relationship_obj) = relationship.as_object() {
				let relation_type = relationship_obj.get("type").as_string()?.read();
				if relation_type == "cover_art" {
					let cover_attributes = relationship_obj.get("attributes").as_object()?;
					cover_file = cover_attributes.get("fileName").as_string()?.read();
					break;
				}
			}
		}
	}

	let mut cover = String::from("https://uploads.mangadex.org/covers/");
	if cover_file.is_empty() {
		cover = cover_file;
	} else {
		cover.push_str(&id);
		cover.push('/');
		cover.push_str(&cover_file);
		cover.push_str(&defaults_get("coverQuality").as_string()?.read());
	}

	Ok(Manga {
		id,
		cover,
		title,
		author: String::new(),
		artist: String::new(),
		description: String::new(),
		url: String::new(),
		categories: Vec::new(),
		status: MangaStatus::Unknown,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Default,
	})
}

// Parse complete manga info
pub fn parse_full_manga(manga_object: ObjectRef) -> Result<Manga> {
	let attributes = manga_object.get("attributes").as_object()?;
	let id = manga_object.get("id").as_string()?.read();

	// Title
	let titles = attributes.get("title").as_object()?;
	let title = get_md_localized_string(titles);

	// Cover, author, artist
	let mut cover_file: String = String::new();
	let mut author: String = String::new();
	let mut artist: String = String::new();

	if let Ok(relationships) = manga_object.get("relationships").as_array() {
		for relationship in relationships {
			let relationship_obj = relationship.as_object()?;
			let relation_type = relationship_obj.get("type").as_string()?.read();
			if let Ok(relationship_attributes) = relationship_obj.get("attributes").as_object() {
				if relation_type == "cover_art" {
					cover_file = relationship_attributes.get("fileName").as_string()?.read();
				} else if relation_type == "author" {
					author = relationship_attributes.get("name").as_string()?.read();
				} else if relation_type == "artist" {
					artist = relationship_attributes.get("name").as_string()?.read();
				}
			}
		}
	}

	let mut cover = String::from("https://uploads.mangadex.org/covers/");
	cover.push_str(&id);
	cover.push('/');
	cover.push_str(&cover_file);
	cover.push_str(&defaults_get("coverQuality").as_string()?.read());

	// Description
	let description = match attributes.get("description").as_object() {
		Ok(descriptions) => get_md_localized_string(descriptions),
		Err(_) => String::new(),
	};

	// URL
	let mut url = String::from("https://mangadex.org/title/");
	url.push_str(&id);

	// Tags
	let categories = if let Ok(tags) = attributes.get("tags").as_array() {
		let mut ret = Vec::with_capacity(tags.len());
		for tag in tags {
			let tag_obj = tag.as_object()?;
			let tag_attributes = tag_obj.get("attributes").as_object()?;
			let names = tag_attributes.get("name").as_object()?;
			ret.push(get_md_localized_string(names));
		}
		ret
	} else {
		Vec::new()
	};

	// Status
	let status_string = match attributes.get("status").as_string() {
		Ok(status) => status.read(),
		Err(_) => String::new(),
	};
	let status = match status_string.as_str() {
		"ongoing" => MangaStatus::Ongoing,
		"completed" => MangaStatus::Completed,
		"hiatus" => MangaStatus::Hiatus,
		"cancelled" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	};

	// Content rating
	let nsfw = match attributes.get("contentRating").as_string() {
		Ok(string) => match string.read().as_str() {
			"suggestive" => MangaContentRating::Suggestive,
			"erotica" => MangaContentRating::Nsfw,
			"pornographic" => MangaContentRating::Nsfw,
			_ => MangaContentRating::Safe,
		},
		Err(_) => MangaContentRating::Safe,
	};

	// Viewer
	let viewer = match attributes.get("originalLanguage").as_string() {
		Ok(string) => match string.read().as_str() {
			"ja" => MangaViewer::Rtl,
			"zh" => MangaViewer::Scroll,
			"ko" => MangaViewer::Scroll,
			_ => MangaViewer::Rtl,
		},
		Err(_) => MangaViewer::Rtl,
	};

	Ok(Manga {
		id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories,
		status,
		nsfw,
		viewer,
	})
}

// Parse chapter info
pub fn parse_chapter(chapter_object: ObjectRef) -> Result<Chapter> {
	let attributes = chapter_object.get("attributes").as_object()?;

	let date_updated = attributes
		.get("publishAt")
		.as_date("yyyy-MM-dd'T'HH:mm:ss+ss:ss", None, None)
		.unwrap_or(-1.0);

	// Fix for Skittyblock/aidoku-community-sources#25
	let ext_url = attributes.get("externalUrl");
	if ext_url.is_none() || ext_url.as_string().is_ok() || date_updated > current_date() {
		return Err(aidoku::error::AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		});
	}

	let id = chapter_object.get("id").as_string()?.read();
	let title = attributes
		.get("title")
		.as_string()
		.unwrap_or_else(|_| StringRef::from(""))
		.read();

	let volume = attributes.get("volume").as_float().unwrap_or(-1.0) as f32;
	let chapter = attributes.get("chapter").as_float().unwrap_or(-1.0) as f32;

	let mut scanlator = String::new();

	if let Ok(relationships) = chapter_object.get("relationships").as_array() {
		for relationship in relationships {
			let relationship_object = relationship.as_object()?;
			let relationship_type = relationship_object.get("type").as_string()?.read();
			if relationship_type == "scanlation_group" {
				if let Ok(relationship_attributes) =
					relationship_object.get("attributes").as_object()
				{
					scanlator = relationship_attributes.get("name").as_string()?.read();
				}
				break;
			}
		}
	}

	let mut url = String::from("https://mangadex.org/chapter/");
	url.push_str(&id);

	let lang = attributes
		.get("translatedLanguage")
		.as_string()
		.unwrap_or_else(|_| StringRef::from("en"))
		.read();

	Ok(Chapter {
		id,
		title,
		volume,
		chapter,
		date_updated,
		scanlator,
		url,
		lang,
	})
}
