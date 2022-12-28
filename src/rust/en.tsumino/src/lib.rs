#![no_std]
use aidoku::{
    error::Result,
    prelude::*,
    std::net::HttpMethod,
    std::net::Request,
    std::{String, Vec},
    Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult,
    MangaStatus, MangaViewer, Page,
};
extern crate alloc;
use alloc::string::ToString;
mod helper;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    aidoku::prelude::println!("here");
    let base_url = String::from("https://www.tsumino.com");
    let mut sort = String::from("Newest");
    let mut tags = String::new();
    let mut i = 0;
    for filter in filters {
        match filter.kind {
            FilterType::Genre => {
                let tpe: i32 = 1;
                tags.push_str(&helper::urlencode(format!("&Tags[{}][Type]={}", i, tpe)));
                tags.push_str(&helper::urlencode(format!(
                    "&Tags[{}][Text]={}",
                    i, filter.name
                )));
                match filter.value.as_int().unwrap_or(-1) {
                    0 => tags.push_str(&helper::urlencode(format!("&Tags[{}][Exclude]=true", i))),
                    1 => tags.push_str(&helper::urlencode(format!("&Tags[{}][Exclude]=false", i))),
                    _ => continue,
                }
            }
            FilterType::Sort => {
                let value = match filter.value.as_object() {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                let index = value.get("index").as_int().unwrap_or(0) as i32;
                let option = match index {
                    0 => "Newest",
                    1 => "Oldest",
                    2 => "Alpabetical",
                    3 => "Rating",
                    4 => "Pages",
                    5 => "Views",
                    6 => "Random",
                    7 => "Comments",
                    8 => "Popularity",
                    _ => continue,
                };
                sort = String::from(option)
            }
            _ => continue,
        }
        i += 1;
    }
    aidoku::prelude::println!("tags: {}", tags);

    let mut url = String::from(base_url + "/search/operate/");
    url.push_str("?PageNumber=");
    url.push_str(&helper::urlencode(page.to_string()));
    url.push_str("&Sort=");
    url.push_str(&helper::urlencode(sort));
    url.push_str(&tags);
    aidoku::prelude::println!("url: {}", url);

    let request = Request::new(&url, HttpMethod::Get).header("User-Agent", "Aidoku");
    let json = request.json()?.as_object()?;
    let data = json.get("data").as_array()?;
    aidoku::prelude::println!("data: {}", data.len());

    let mut manga_arr: Vec<Manga> = Vec::new();
    let total: i32;
    for manga in data {
        aidoku::prelude::println!("here");
        let obj = manga.as_object()?;
        let md = obj.get("entry").as_object()?;
        let id = helper::get_id(md.get("id"))?;
        let title = md.get("title").as_string()?.read();
        let cover = md.get("thumbnailUrl").as_string()?.read();
        manga_arr.push(Manga {
            id,
            cover,
            title,
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
    aidoku::prelude::println!("manga_arr: {}", manga_arr.len());
    total = json.get("pageCount").as_int().unwrap_or(0) as i32;
    aidoku::prelude::println!("total: {}", total);

    Ok(MangaPageResult {
        manga: manga_arr,
        has_more: page < total,
    })
}

#[get_manga_listing]
fn get_manga_listing(_: Listing, _: i32) -> Result<MangaPageResult> {
    todo!()
}

#[get_manga_details]
fn get_manga_details(_: String) -> Result<Manga> {
    todo!()
}

#[get_chapter_list]
fn get_chapter_list(_: String) -> Result<Vec<Chapter>> {
    todo!()
}

#[get_page_list]
fn get_page_list(_: String, _: String) -> Result<Vec<Page>> {
    todo!()
}

#[modify_image_request]
fn modify_image_request(_: Request) {
    todo!()
}

#[handle_url]
fn handle_url(_: String) -> Result<DeepLink> {
    todo!()
}
