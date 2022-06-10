#![no_std]
use aidoku::{
    error::Result,
    prelude::*,
    std::{String, Vec},
    Chapter, Listing, Manga, MangaPageResult, Page, std::net::Request, Filter
};

pub mod parser;
pub mod helper;

const BASE_URL: &'static str = "https://readm.org";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    parser::parse_manga_list(String::from(BASE_URL), filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
    let url = match listing.name.as_str() {
        "Latest" => format!("{}/latest-releases/{}", BASE_URL, page),
        "Popular" => format!("{}/popular-manga/{}", BASE_URL,page),
        _ => String::from(BASE_URL),
    };
    parser::parse_manga_listing(String::from(BASE_URL),String::from(url), String::from(listing.name))
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
    parser::parse_manga_details(String::from(BASE_URL), id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
    parser::parse_chapter_list(String::from(BASE_URL), String::from("MMM dd,yyyy HH:mm"),id)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
    parser::parse_page_list(String::from(BASE_URL),id)

}

#[modify_image_request] 
fn modify_image_request(request: Request) {
	parser::modify_image_request(String::from(BASE_URL), request)
}