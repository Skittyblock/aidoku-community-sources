#![no_std]
pub mod helper;
use aidoku::{
    prelude::*, error::Result, std::String, std::Vec, std::net::Request,
    Filter, Listing, Manga, MangaPageResult, Chapter, DeepLink, FilterType, Page
};
use wpcomics_template::{template,helper::urlencode};
use crate::helper::*;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    let mut title: String = String::new();
    let mut genre: String = String::new();
    for filter in filters {
        match filter.kind {
            FilterType::Title => {
                title = urlencode(filter.value.as_string()?.read());
            },
            _ => {
                match filter.name.as_str() {
                    "Genre" => {
                        genre = get_tag_id(filter.value.as_int().unwrap_or(0));
                        println!("genre: {}", genre);
                    },
                    _ => continue,
                }
            },
        }
    }
    template::get_manga_list(
        get_search_url(
            String::from("https://xoxocomics.com"),
            title,
            genre,
            page,
        ),
        String::from("li > a[rel=next]"),
        trunc_trailing_comic
    )
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
    template::get_manga_listing(String::from("https://xoxocomics.com"), listing, String::from("li > a[rel=next]"), listing_map, trunc_trailing_comic, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
    template::get_manga_details(id, status_map, trunc_trailing_comic)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
    template::get_chapter_list(id, trunc_trailing_comic, true, String::from("div.col-xs-3"), convert_time)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
    template::get_page_list(id, String::from("/all"), |url| {
        return url;
    })
}

#[modify_image_request] 
fn modify_image_request(request: Request) {
    template::modify_image_request(
        String::from("https://xoxocomics.com"),
        String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.0.4951.54 Safari/537.36 Edg/101.0.1210.39"),
        request,
    )
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
    template::handle_url(url, status_map, trunc_trailing_comic)
}
