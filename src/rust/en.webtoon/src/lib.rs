#![no_std]
extern crate alloc;


use aidoku::{Chapter, error::Result, Filter, FilterType, Manga, MangaPageResult, Page, prelude::*, std::net::HttpMethod, std::net::Request, std::String, std::Vec};

use crate::parser::urlencode;

mod parser;

const BASE_URL: &str = "https://webtoons.com";
const MOBILE_BASE_URL: &str = "https://m.webtoons.com";
const MOBILE_USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 13_2_3 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/13.0.3 Mobile/15E148 Safari/604.1";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, _page: i32) -> Result<MangaPageResult> {
    for filter in filters {
        match filter.kind {
            FilterType::Title => {
                let url = format!("{}/en/search?keyword={}", BASE_URL, urlencode(filter.value.as_string()?.read()));
                let html = Request::new(url.as_str(), HttpMethod::Get).html();

                let mut manga = parser::parse_search(&html, false);
                manga.append(&mut parser::parse_search(&html, true));

                return Ok(MangaPageResult {
                    manga,
                    has_more: false,
                });
            }
            _ => {}
        }
    }

    let url = format!("{}/en/top", BASE_URL);
    let html = Request::new(url.as_str(), HttpMethod::Get).html();

    Ok(MangaPageResult {
        manga: parser::parse_manga_list(html),
        has_more: false,
    })
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
    let url = format!("{}/en/{}", BASE_URL, &manga_id);
    let html = Request::new(url.clone().as_str(), HttpMethod::Get).html();
    return parser::parse_manga(html, manga_id);
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
    let url = format!("{}/en/{}", MOBILE_BASE_URL, &manga_id);
    let html = Request::new(url.as_str(), HttpMethod::Get)
        .header("User-Agent", MOBILE_USER_AGENT)
        .html();

    return parser::get_chapter_list(html, manga_id);
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
    let [manga_id, chapter_id]: [&str; 2] = <[&str; 2]>::try_from(id.split("|").collect::<Vec<&str>>()).unwrap();
    let url = format!("{}/en/{}&episode_no={}", BASE_URL, &manga_id, &chapter_id)
        .replace("list", format!("ep{}/viewer", chapter_id).as_str());

    let html = Request::new(url.clone().as_str(), HttpMethod::Get).html();
    return parser::get_page_list(html);
}

#[modify_image_request]
fn modify_image_request(request: Request) {
    request
        .header("Cookie", "pagGDPR=true;")
        .header("Referer", format!("{}/", BASE_URL).as_str())
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.0.5005.124 Safari/537.36 Edg/102.0.1245.44");
}