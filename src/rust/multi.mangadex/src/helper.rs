use aidoku::{
    error::Result, std::String, std::Vec, std::ObjectRef, Manga, MangaStatus, MangaContentRating, MangaViewer, Chapter,
    std::defaults::defaults_get,
};

// Parse manga with title and cover
pub fn parse_basic_manga(manga_object: ObjectRef) -> Result<Manga> {
    let attributes = manga_object.get("attributes").as_object()?;
    let id = manga_object.get("id").as_string()?.read();

    // Title
    let titles = attributes.get("title").as_object()?;
    let title = match titles.get("en").as_string() { // try for english title
        Ok(title) => title.read(),
        Err(_) => match titles.get("ja").as_string() { // try for japanese (possibly romaji)
            Ok(title) => title.read(),
            Err(_) => match titles.values().get(0).as_string() { // get first title instead
                Ok(title) => title.read(),
                Err(_) => String::new(),
            },
        },
    };

    // Cover
    let mut cover_file: String = String::new();
    
    let relationships = manga_object.get("relationships").as_array()?;
    for relationship in relationships {
        let relationship_obj = relationship.as_object()?;
        let relation_type = relationship_obj.get("type").as_string()?.read();
        if relation_type == "cover_art" {
            let cover_attributes = relationship_obj.get("attributes").as_object()?;
            cover_file = cover_attributes.get("fileName").as_string()?.read();
            break;
        }
    }

    let mut cover = String::from("https://uploads.mangadex.org/covers/");
    cover.push_str(&id);
    cover.push_str("/");
    cover.push_str(&cover_file);
    cover.push_str(&defaults_get("coverQuality").as_string()?.read());

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
    let title = match titles.get("en").as_string() { // try for english title
        Ok(title) => title.read(),
        Err(_) => match titles.get("ja").as_string() { // try for japanese (possibly romaji)
            Ok(title) => title.read(),
            Err(_) => match titles.values().get(0).as_string() { // get first title instead
                Ok(title) => title.read(),
                Err(_) => String::new(),
            }
        },
    };

    // Cover, author, artist
    let mut cover_file: String = String::new();
    let mut author: String = String::new();
    let mut artist: String = String::new();
    
    let relationships = manga_object.get("relationships").as_array()?;
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

    let mut cover = String::from("https://uploads.mangadex.org/covers/");
    cover.push_str(&id);
    cover.push_str("/");
    cover.push_str(&cover_file);
    cover.push_str(&defaults_get("coverQuality").as_string()?.read());

    // Description
    let description = match attributes.get("description").as_object() {
		Ok(descriptions) => match descriptions.get("en").as_string() { // try for english desc
			Ok(desc) => desc.read(),
			Err(_) => match descriptions.values().get(0).as_string() { // get first desc instead
				Ok(desc) => desc.read(),
				Err(_) => String::new(),
			}
		},
		Err(_) => String::new(),
	};

    // URL
    let mut url = String::from("https://mangadex.org/title/");
    url.push_str(&id);

    // Tags
    let mut categories: Vec<String> = Vec::new();
    if let Ok(tags) = attributes.get("tags").as_array() {
		for tag in tags {
			let tag_obj = tag.as_object()?;
			let tag_attributes = tag_obj.get("attributes").as_object()?;
			let names = tag_attributes.get("name").as_object()?;
			categories.push( match names.get("en").as_string() {
				Ok(name) => name.read(),
				Err(_) => match names.values().get(0).as_string() {
					Ok(name) => name.read(),
					Err(_) => String::new(),
				}
			});
		}
	}

    // Status
    let status_string = attributes.get("status").as_string()?.read();
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

	let pages = attributes.get("pages").as_int()?;

	if pages <= 0 { 
		return Err(aidoku::error::AidokuError { reason: aidoku::error::AidokuErrorKind::Unimplemented });
	}

	let id = chapter_object.get("id").as_string()?.read();
	let title = attributes.get("title").as_string()?.read();

	let volume = match attributes.get("volume").as_float() {
		Ok(volume) => volume as f32,
		Err(_) => -1.0,
	};
	
	let chapter = match attributes.get("chapter").as_float() {
		Ok(chapter) => chapter as f32,
		Err(_) => -1.0,
	};

	let date_updated = match attributes.get("publishAt").as_date("yyyy-MM-dd'T'HH:mm:ss+ss:ss") {
		Ok(date) => date,
		Err(_) => -1.0,
	};

	let mut scanlator = String::new();

	let relationships = chapter_object.get("relationships").as_array()?;
	for relationship in relationships {
		let relationship_object = relationship.as_object()?;
		let relationship_type = relationship_object.get("type").as_string()?.read();
		if relationship_type == "scanlation_group" {
			let relationship_attributes = relationship_object.get("attributes").as_object()?;
			scanlator = relationship_attributes.get("name").as_string()?.read();
			break;
		}
	}

	let mut url = String::from("https://mangadex.org/chapter/");
	url.push_str(&id);

	let lang = attributes.get("translatedLanguage").as_string()?.read();

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
