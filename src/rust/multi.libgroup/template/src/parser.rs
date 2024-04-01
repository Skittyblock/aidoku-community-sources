use aidoku::prelude::format;
use aidoku::std::String;
use aidoku::MangaContentRating;
use aidoku::{std::ObjectRef, Manga, MangaPageResult};
use aidoku::error::Result;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::helpers::{id_to_status, siteid_to_domain};
use crate::helpers::SiteId;
extern crate alloc;


pub fn parse_manga_list(js: ObjectRef, site: &SiteId) -> Result<MangaPageResult> {
    let has_more = js.get("meta").as_object()?.get("has_next_page").as_bool()?;
    let mangas = js.get("data").as_array()?;
    let mut arr_manga: Vec<Manga> = Vec::new();

    let domain = siteid_to_domain(site);
    for data in mangas {
        if let Ok(data_obj) = data.as_object() {
            let title = match data_obj.get("eng_name").as_string() {
                Ok(x) => x.read(),
                Err(_) => continue
            };
            let id = match data_obj.get("slug_url").as_string() { 
                Ok(x) => x.to_string(),
                Err(_) => continue
            };
            let cover =  match data_obj.get("cover").as_object()?.get("default").as_string() {
                Ok(x) => x.read(),
                Err(_) => continue
            };
            let url = match data_obj.get("slug_url").as_string() {
                Ok(x) => format!("https://{}/ru/manga/{}", domain, x.read()),
                Err(_) => continue
            };
            let status = match data_obj.get("status").as_object()?.get("id").as_int() {
                Ok(x) => id_to_status(x),
                Err(_) => continue
            };

            arr_manga.push( Manga {
                id: id,
                cover: cover,
                title: title,
                author: String::new(),
                artist: String::new(),
                description: String::new(),
                url: url,
                categories: Vec::new(),
                status: status,
                nsfw: MangaContentRating::Safe,
                viewer: aidoku::MangaViewer::Rtl
            })
        }
    }

    Ok(MangaPageResult {
        manga: arr_manga,
        has_more: has_more
    })
}