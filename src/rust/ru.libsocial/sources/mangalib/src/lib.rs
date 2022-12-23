#![no_std]

extern crate alloc;

use alloc::borrow::ToOwned;
use aidoku::{error::Result, prelude::*, std::{String, Vec}, Chapter, Filter, Listing, Manga, MangaPageResult, Page, FilterType};
use aidoku::helpers::uri::QueryParameters;
use aidoku::std::{print, StringRef};
use libsocial_aidoku::{LibGroup, parser};

static INSTANCE: LibGroup = LibGroup {
    base_url: "https://mangalib.me",
};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    let mut buffer = itoa::Buffer::new();
    let mut parameters = QueryParameters::new();
    parameters.push("page", Some(buffer.format(page)));
    for filter in filters {
        match filter.kind {
            FilterType::Check => {
                let Ok(value) = filter.value.as_int() else {
                    continue;
                };
                let Ok(param) = filter.object.get("id").as_string() else {
                    continue;
                };
                let id = param.read();
                let Some((key, id)) = id.split_once('=') else {
                    continue;
                };
                match key {
                    "type" => {
                        if value == 1 {
                            parameters.push(&format!("{}[]", key), Some(&id.to_owned()))
                        }
                    }
                    "format" => {
                        match value {
                            0 => {
                                parameters.push(&format!("{}[exclude][]", key), Some(&id.to_owned()))
                            }
                            1 => {
                                parameters.push(&format!("{}[include][]", key), Some(&id.to_owned()))
                            }
                            _ => {}
                        }
                    }
                    "status" => {
                        if value == 1 {
                            parameters.push(&format!("{}[]", key), Some(&id.to_owned()))
                        }
                    },
                    "manga_status" => {
                        if value == 1 {
                            parameters.push(&format!("{}[]", key), Some(&id.to_owned()))
                        }
                    }
                    _ => {}
                }
            }
            FilterType::Sort => {
                let value = match filter.value.as_object() {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                let index = value.get("index").as_int().unwrap_or(0);
                let ascending = value.get("ascending").as_bool().unwrap_or(false);
                parameters.push("", Some(match index {
                    0 => "rate",
                    1 => "name",
                    2 => "views",
                    3 => "created_at",
                    4 => "last_chapter_at",
                    5 => "chap_count",
                    _ => {
                        continue;
                    }
                }));
                parameters.push("dir", Some(if ascending { "asc" } else { "desc" }));
            }
            FilterType::Title => {
                if let Ok(value) = filter.value.as_string() {
                    parameters.push("name", Some(&value.read()));
                }
            }
            FilterType::Genre => {
                let Ok(value) = filter.value.as_int() else {
                    continue;
                };
                let Ok(param) = filter.object.get("id").as_string() else {
                    continue;
                };
                let id = param.read();
                let Some((key, id)) = id.split_once('=') else {
                    continue;
                };
                match value {
                    0 => {
                        parameters.push(&format!("{}[exclude][]", key), Some(&id.to_owned()))
                    }
                    1 => {
                        parameters.push(&format!("{}[include][]", key), Some(&id.to_owned()))
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    let result = INSTANCE.get_manga_list_request(parameters)?
        .json()?
        .as_object()?;
    println!("Get manga {}", result.len());
    parser::manga_list_parse(INSTANCE.base_url, result)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
    if listing.name == "Обновления" {
        let mut buffer = itoa::Buffer::new();
        let mut parameters = QueryParameters::new();
        parameters.push("page", Some(buffer.format(page)));
        parameters.push("dir", Some("desc"));
        parameters.push("sort", Some("last_chapter_at"));
        let result = INSTANCE.get_manga_list_request(parameters)?
            .json()?
            .as_object()?;
        parser::manga_list_parse(INSTANCE.base_url, result)
    } else {
        get_manga_list(Vec::new(), page)
    }
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