use alloc::string::String;
use aidoku::{MangaPageResult, error::Result, Manga, std::Vec, prelude::*};
use aidoku::std::ObjectRef;

pub fn manga_list_parse<T: AsRef<str>>(base_url: T, response: ObjectRef) -> Result<MangaPageResult> {
    let items = response.get("items").as_object()?;
    let data = items.get("data").as_array()?;
    let manga = data
        .filter_map(move |manga| match manga.as_object() {
            Ok(obj) => manga_parse(&base_url, obj).ok(),
            Err(_) => None,
        })
        .collect::<Vec<Manga>>();

    let has_more = items.get("next_page_url").is_some();

    Ok(MangaPageResult { manga, has_more })
}

pub fn manga_parse<T: AsRef<str>>(base_url:T, response: ObjectRef) -> Result<Manga> {
    let id = response.get("id").as_int()?;
    let title = response.get("rus_name").as_string()?;
    let cover = response.get("coverImage").as_string()?;
    let slug = response.get("slug").as_string()?;
    Ok(
        Manga {
            id: format!("{}", id),
            cover: format!("{}/{}", base_url.as_ref(), cover.read()),
            title: title.read(),
            author: String::from(""),
            artist: String::from(""),
            description: String::from(""),
            url: format!("{}/{}", base_url.as_ref(), slug.read()),
            categories: Vec::new(),
            status: Default::default(),
            nsfw: Default::default(),
            viewer: Default::default(),
        }
    )
}