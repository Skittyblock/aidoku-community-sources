#![no_std]
use aidoku::{
    error::Result,
    prelude::*,
    std::{String, Vec},
    Chapter, Filter, Listing, Manga, MangaPageResult, Page,
};
use libsocial_aidoku::{LibGroup, parser};

static INSTANCE: LibGroup = LibGroup {
    base_url: "https://mangalib.me",
};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    let result = INSTANCE.get_manga_list_request(filters, page)?
        .json()?
        .as_object()?;
    println!("Get manga {}", result.len());
    parser::manga_list_parse(INSTANCE.base_url, result)
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