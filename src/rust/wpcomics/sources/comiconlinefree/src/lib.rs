#![no_std]
mod helper;
use aidoku::{
    error::Result,
    prelude::*,
    std::{net::Request, String, Vec},
    Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, MangaViewer, Page,
};
use helper::{convert_time, listing_mapping, status_map, trunc_trailing_comic};
use wpcomics_template::{
    template::{self, *},
    helper::urlencode,
};

pub static SELECTORS: Selectors = Selectors {
    next_page: "div.general-nav > a:contains(Next)",
    manga_cell: ".manga-box",
    manga_cell_title: "div.mb-right h3 > a",
    manga_cell_url: "div.mb-right h3 > a",
    manga_cell_image: "a.image > img",

    manga_details_title: "h1.manga-title",
    manga_details_title_transformer: trunc_trailing_comic,
    manga_details_cover: "div.manga-image > img",
    manga_details_author: "div.manga-details > table > tbody > tr > td:contains(Author) + td",
    manga_details_author_transformer: |title| String::from(title.trim()),
    manga_details_description: "div.manga-desc > p.pdesc",
    manga_details_tags: "div.manga-details > table > tbody > tr > td:contains(Author) + td",
    manga_details_tags_splitter: ", ",
    manga_details_status: "div.manga-details > table > tbody > tr > td:contains(Status) + td",
    manga_details_status_transformer: |title| String::from(title.trim()),
    manga_details_chapters: "ul.basic-list > li",

    manga_viewer_page: "div.chapter-container > img",

    chapter_anchor_selector: "a.ch-name",
    chapter_date_selector: "a.ch-name + span",
};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    let mut included_tags: Vec<String> = Vec::new();
    let mut excluded_tags: Vec<String> = Vec::new();
    let mut title: String = String::new();
    let mut completed: String = String::new();
    for filter in filters {
        match filter.kind {
            FilterType::Title => {
                title = urlencode(filter.value.as_string()?.read());
            }
            FilterType::Genre => match filter.value.as_int().unwrap_or(-1) {
                0 => excluded_tags.push(String::from(&filter.name)),
                1 => included_tags.push(String::from(&filter.name)),
                _ => continue,
            },
            _ => {
                match filter.name.as_str() {
                    "Status" => {
                        let completed_idx = filter.value.as_int().unwrap_or(-1);
                        match completed_idx {
                            1 => completed = String::from("ONG"),
                            2 => completed = String::from("CMP"),
                            _ => continue,
                        }
                    },
                    _ => continue,
                }
            }
        }
    }
    template::get_manga_list(
        helper::get_search_url(
            String::from("https://comiconlinefree.net"), 
            title, 
            included_tags, 
            excluded_tags, 
            completed, 
            page
        ),
        &SELECTORS
    )
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
    #[allow(unused_assignments)]
    let mut url = String::new();
    if page > 1 {
        url = format!("https://comiconlinefree.net/{}/{page}", listing_mapping(listing.name));
    } else {
        url = format!("https://comiconlinefree.net/{}/", listing_mapping(listing.name));
    }
    template::get_manga_list(url, &SELECTORS)
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
    template::get_page_list(id, &SELECTORS, String::from("/full"), |url| {
        return url;
    })
}

#[modify_image_request]
fn modify_image_request(request: Request) {
    template::modify_image_request(
        String::from("https://comiconlinefree.net"),
        String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.0.4951.54 Safari/537.36 Edg/101.0.1210.39"),
        request,
    )
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
    template::handle_url(url, &SELECTORS, MangaViewer::Ltr, status_map)
}
