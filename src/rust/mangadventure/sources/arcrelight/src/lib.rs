#![no_std]

use mangadventure_template::*;

static SOURCE: MangAdventure = MangAdventure {
	base_url: "https://arc-relight.com",
	language: "en",
};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	SOURCE.get_manga_list(filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	SOURCE.get_manga_listing(listing, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	SOURCE.get_manga_details(id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	SOURCE.get_chapter_list(id)
}

#[get_page_list]
fn get_page_list(_: String, id: String) -> Result<Vec<Page>> {
	SOURCE.get_page_list(id)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	SOURCE.handle_url(url)
}
