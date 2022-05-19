#![no_std]
pub mod helper;
use aidoku::{
    prelude::*, error::Result, std::String, std::Vec, std::net::Request,
    Filter, Listing, Manga, MangaPageResult, Chapter, DeepLink, FilterType, Page, MangaViewer
};
use wpcomics_template::{
    template,
    template::Selectors,
    helper::urlencode
};
use crate::helper::*;

static SELECTORS: Selectors = Selectors {
    next_page: "li > a[rel=next]",
    manga_cell: "div.items > div.row > div.item > figure.clearfix",
    manga_cell_title: "figcaption > h3 > a",
    manga_cell_url: "div.image > a",
    manga_cell_image: "div.image > a > img",

    manga_details_title: "h1.title-detail",
    manga_details_title_transformer: trunc_trailing_comic,
    manga_details_cover: "div.col-image > img",
    manga_details_author: "ul.list-info > li.author > p.col-xs-8",
    manga_details_author_transformer: |title| title,
    manga_details_description: "div.detail-content > p",
    manga_details_tags: "li.kind.row > p.col-xs-8",
    manga_details_status: "li.status.row > p.col-xs-8",
    manga_details_status_transformer: |title| title,
    manga_details_chapters: "div.list-chapter > nav > ul > li",

    manga_viewer_page: "div.page-chapter > img",

    chapter_anchor_selector: "div.chapter > a",
    chapter_date_selector: "div.col-xs-3"
};

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
        &SELECTORS,
    )
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
    template::get_manga_listing(
        String::from("https://xoxocomics.com"), 
        listing, 
        &SELECTORS,
        listing_map, 
        page
    )
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
    template::get_manga_details(id, &SELECTORS, MangaViewer::Ltr, status_map)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
    template::get_chapter_list(id, &SELECTORS, true, convert_time)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
    template::get_page_list(id, &SELECTORS, String::from("/all"), |url| {
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
    template::handle_url(url, &SELECTORS, MangaViewer::Ltr, status_map)
}
