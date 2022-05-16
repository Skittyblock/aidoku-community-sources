use aidoku::{
	error::Result, std::String, std::Vec, std::ObjectRef,
	Manga, MangaContentRating, MangaViewer, Chapter,
};

use crate::helper::{status_map, i32_to_string, extract_f32_from_string};

pub fn parse_manga(manga_object: ObjectRef) -> Result<Manga> {
    let id = manga_object.get("id").as_int().unwrap_or(-1);
    let title = manga_object.get("originalName").as_string()?.read();
    let cover = manga_object.get("thumbnail").as_string()?.read();
    let authors = manga_object.get("author").as_array()?;
    let author_string = authors.map(|author| {
        let author_object = author.as_object()?;
        return Ok(author_object.get("name").as_string()?.read());
    })
        .map(|a: Result<String>| a.unwrap_or(String::from("")))
        .collect::<Vec<String>>()
        .join(", ");
    let mut url = String::from("https://yurineko.net/manga/");
    url.push_str(&i32_to_string(id as i32));

    let tags = manga_object.get("tag").as_array()?;
    let tags_array = tags.map(|tag| {
        let tag_object = tag.as_object()?;
        Ok(tag_object.get("name").as_string()?.read())
    })
        .map(|a: Result<String>| a.unwrap_or(String::from("")))
        .collect::<Vec<String>>();

    let mut content_rating: MangaContentRating = MangaContentRating::Safe;
    let mut view_direction: MangaViewer = MangaViewer::Rtl;
    for tag in &tags_array {
        if tag.contains("sex") || tag.contains("NSFW") || tag.contains("R18") {
            content_rating = MangaContentRating::Nsfw;
        }
        else if tag.contains("Ecchi") {
            content_rating = MangaContentRating::Suggestive;
        }
        if tag.contains(">") || tag.contains("Manhua") || tag.contains("Manhwa") {
            view_direction = MangaViewer::Ltr;
        }
    }
    Ok(Manga {
        id: i32_to_string(id as i32),
        cover,
        title,
        author: author_string,
        artist: String::new(),
        description: String::new(), // todo: parse html,
        url,
        categories: tags_array,
        status: status_map(manga_object.get("status").as_int().unwrap_or(-1)),
        nsfw: content_rating,
		viewer: view_direction,
    })
}

pub fn parse_chapter(scanlator: String, chapter_object: ObjectRef) -> Result<Chapter> {
    let id = chapter_object.get("id").as_int().unwrap_or(-1);
    let manga_id = chapter_object.get("mangaID").as_int().unwrap_or(-1);
    let title = chapter_object.get("name").as_string()?.read();
    let date_string = chapter_object.get("date").as_string()?;
    let chapter_number = extract_f32_from_string(String::from("-"), String::from(&title));

    let date_object = date_string.0.as_date("yyyy-MM-dd'T'HH:mm:ss.SSS'Z'", Some("en_US"), Some("UTC")).unwrap_or(-1.0);

    let mut url = String::from("https://yurineko.net/read/");
    let mut chapter_id = String::new();
    chapter_id.push_str(&i32_to_string(manga_id as i32));
    chapter_id.push_str("/");
    chapter_id.push_str(&i32_to_string(id as i32));
    url.push_str(chapter_id.as_str());

    Ok(Chapter {
        id: chapter_id,
        title,
        volume: -1.0,
        chapter: chapter_number,
        date_updated: date_object,
        scanlator,
        url,
        lang: String::from("vi")
    })
}
