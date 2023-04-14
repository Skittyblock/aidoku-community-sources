#![no_std]
use aidoku::{
    error::Result,
    prelude::*,
    std::{
        net::{HttpMethod, Request},
        String, Vec,
    },
    Chapter, Filter, Manga, MangaPageResult, Page,
};

mod parser;

const BASE_URL: &str = "https://www.baozimh.com";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    let mut url = String::new();
    parser::get_filtered_url(filters, page, &mut url);

    let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
    if url.contains("https://www.baozimh.com/classify") {
        return parser::parse_home_page(html);
    }
    parser::parse_search_page(html)
}

#[get_manga_details]
fn get_manga_details(manga: String) -> Result<Manga> {
    let url = format!("{}/comic/{}", BASE_URL, manga);
    let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
    parser::get_manga_details(html, manga)
}

#[get_chapter_list]
fn get_chapter_list(manga: String) -> Result<Vec<Chapter>> {
    let url = format!("{}/comic/{}", BASE_URL, manga);
    let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
    parser::get_chapter_list(html)
}

#[get_page_list]
fn get_page_list(manga: String, chapter: String) -> Result<Vec<Page>> {
    let url = format!("{}/comic/chapter/{}/0_{}.html", BASE_URL, manga, chapter);
    let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
    parser::get_page_list(html)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
    request.header("Referer", BASE_URL);
}

// #[handle_url]
// fn handle_url(_: String) -> Result<DeepLink> {
//     todo!()
// }
