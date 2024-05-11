use aidoku::{
	error::Result,
	std::{defaults::defaults_get, ObjectRef, String, Vec},
	Chapter, Manga, MangaContentRating, MangaStatus, MangaViewer,
};

fn get_md_localized_string(obj: ObjectRef) -> String {
	let use_preferred_lang = match defaults_get("usePreferredLanguage") {
		Ok(value) => value.as_bool().unwrap_or(false),
		Err(_) => false,
	};
	let language = if use_preferred_lang {
		match defaults_get("languages") {
			Ok(langs) => match langs.as_array() {
				Ok(langs) => match langs.get(0).as_string() {
					Ok(lang) => lang.read(),
					Err(_) => String::from("en"),
				},
				Err(_) => return String::from("en"),
			},
			Err(_) => String::from("en"),
		}
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
				// Screw it, get first value
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
	let title = attributes
		.get("title")
		.as_object()
		.map(get_md_localized_string)
		.unwrap_or_default();

	// Cover
	let mut cover_file: String = String::new();

	if let Ok(relationships) = manga_object.get("relationships").as_array() {
		for relationship in relationships {
			if let Ok(relationship_obj) = relationship.as_object()
				&& let Ok(relation_type) = relationship_obj.get("type").as_string()
				&& let Ok(attribs) = relationship_obj.get("attributes").as_object()
				&& relation_type.read() == "cover_art"
			{
				cover_file = attribs.get("fileName").as_string().map(|v| v.read()).unwrap_or_default();
				break;
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
		if let Ok(cover_quality) = defaults_get("coverQuality") {
			cover.push_str(
				&cover_quality
					.as_string()
					.map(|v| v.read())
					.unwrap_or_default(),
			);
		}
	}

	Ok(Manga {
		id,
		cover,
		title,
		..Default::default()
	})
}

// Parse complete manga info
pub fn parse_full_manga(manga_object: ObjectRef) -> Result<Manga> {
	let attributes = manga_object.get("attributes").as_object()?;
	let id = manga_object.get("id").as_string()?.read();

	// Title
	let title = attributes
		.get("title")
		.as_object()
		.map(get_md_localized_string)
		.unwrap_or_default();

	// Cover, author, artist
	let mut cover_file: String = String::new();
	let mut author: String = String::new();
	let mut artist: String = String::new();

	if let Ok(relationships) = manga_object.get("relationships").as_array() {
		for relationship in relationships {
			if let Ok(relationship_obj) = relationship.as_object()
				&& let Ok(relation_type) = relationship_obj.get("type").as_string()
				&& let Ok(attribs) = relationship_obj.get("attributes").as_object()
			{
				match relation_type.read().as_str() {
					"cover_art" => cover_file = attribs.get("fileName").as_string().map(|v| v.read()).unwrap_or_default(),
					"author" => author = attribs.get("name").as_string().map(|v| v.read()).unwrap_or_default(),
					"artist" => artist = attribs.get("name").as_string().map(|v| v.read()).unwrap_or_default(),
					_ => continue,
				}
			}
		}
	}

	let mut cover = String::from("https://uploads.mangadex.org/covers/");
	cover.push_str(&id);
	cover.push('/');
	cover.push_str(&cover_file);
	if let Ok(cover_quality) = defaults_get("coverQuality") {
		cover.push_str(
			&cover_quality
				.as_string()
				.map(|v| v.read())
				.unwrap_or_default(),
		);
	}

	// Description
	let description = attributes
		.get("description")
		.as_object()
		.map(get_md_localized_string)
		.unwrap_or_default();

	// URL
	let mut url = String::from("https://mangadex.org/title/");
	url.push_str(&id);

	// Tags
	let categories = attributes
		.get("tags")
		.as_array()
		.map(|tags| {
			tags.filter_map(|tag| {
				if let Ok(obj) = tag.as_object()
			       && let Ok(attribs) = obj.get("attributes").as_object()
			       && let Ok(names) = attribs.get("name").as_object() {
					Some(get_md_localized_string(names))
				} else {
					None
				}
			})
			.collect::<Vec<_>>()
		})
		.unwrap_or_default();

	// Status
	let status_string = attributes
		.get("status")
		.as_string()
		.map(|v| v.read())
		.unwrap_or_default();

	let status = match status_string.as_str() {
		"ongoing" => MangaStatus::Ongoing,
		"completed" => MangaStatus::Completed,
		"hiatus" => MangaStatus::Hiatus,
		"cancelled" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	};

	// Content rating
	let nsfw = match attributes
		.get("contentRating")
		.as_string()
		.map(|v| v.read())
		.unwrap_or_default()
		.as_str()
	{
		"suggestive" => MangaContentRating::Suggestive,
		"erotica" => MangaContentRating::Nsfw,
		"pornographic" => MangaContentRating::Nsfw,
		_ => MangaContentRating::Safe,
	};

	// Viewer
	let viewer = match attributes
		.get("originalLanguage")
		.as_string()
		.map(|v| v.read())
		.unwrap_or_default()
		.as_str()
	{
		"ja" => MangaViewer::Rtl,
		"zh" => MangaViewer::Scroll,
		"ko" => MangaViewer::Scroll,
		_ => MangaViewer::Rtl,
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
		.as_date("yyyy-MM-dd'T'HH:mm:ss+ss:ss", Some("en-US"), Some("UTC"))
		.unwrap_or(-1.0);

	// Fix for Skittyblock/aidoku-community-sources#25
	let ext_url = attributes.get("externalUrl");
	if ext_url.as_string().is_ok()
		|| date_updated > crate::helper::current_date()
	{
		return Err(aidoku::error::AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		});
	}

	let id = chapter_object.get("id").as_string()?.read();
	let mut title = attributes
		.get("title")
		.as_string()
		.map(|v| v.read())
		.unwrap_or_default();

	let volume = attributes.get("volume").as_float().unwrap_or(-1.0) as f32;
	let chapter = attributes.get("chapter").as_float().unwrap_or(-1.0) as f32;

	// As per MangaDex upload guidelines, if the volume and chapter are both null or
	// for serialized entries, the volume is 0 and chapter is null, it's a oneshot.
	// They should have a title of "Oneshot" but some don't, so we'll add it if it's
	// missing.
	if (volume == -1.0 || volume == 0.0) && chapter == -1.0 && title.is_empty() {
		title = String::from("Oneshot");
	}

	let mut uploader = String::new();
	let mut group: Vec<String> = Vec::new();

	if let Ok(relationships) = chapter_object.get("relationships").as_array() {
		for relationship in relationships {
			if let Ok(relationship_object) = relationship.as_object()
				&& let Ok(relation_type) = relationship_object.get("type").as_string()
				&& let Ok(attribs) = relationship_object.get("attributes").as_object()
			{
				if relation_type.clone().read() == "scanlation_group" {
					group.push(attribs.get("name").as_string().map(|v| v.read()).unwrap_or_default());
				} else if relation_type.clone().read() == "user" {
					uploader = attribs.get("username").as_string().map(|v| v.read()).unwrap_or_default();
				}
			}
		}
	}

	// If there's no group, use the uploader as the scanlator
	let scanlator = {
		if group.is_empty() {
			uploader
		} else {
			group.join(", ")
		}
	};

	let mut url = String::from("https://mangadex.org/chapter/");
	url.push_str(&id);

	let lang = attributes
		.get("translatedLanguage")
		.as_string()
		.map(|v| v.read())
		.unwrap_or_else(|_| String::from("en"));

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
