#![no_std]

use aidoku::{
    error::Result,
    prelude::*,
    std::Vec,
    Filter, Listing, MangaPageResult,
};
use mangalib_template::{helpers::SiteId, template::SocialLibSource};

static INSTANCE: SocialLibSource = SocialLibSource {
    site_id: &SiteId::MangaLib
};

#[get_manga_list]
fn get_manga_list(filter: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    INSTANCE.get_manga_list(filter, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
    INSTANCE.get_manga_listing(listing, page)
}