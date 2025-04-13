#![no_std]

use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, Filter, Listing, Manga, MangaPageResult, Page,
};
use mangalib_template::{helpers::is_logged, template::SocialLibSource};

static INSTANCE: SocialLibSource = SocialLibSource {
	site_id: "1",
	domain: "mangalib.me",
	nsfw: &aidoku::MangaContentRating::Safe,
};

#[initialize]
fn initialize() {
	if is_logged() {
		let _ = INSTANCE.refresh_token();
	}
}

#[get_manga_list]
fn get_manga_list(filter: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	INSTANCE.get_manga_list(filter, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	INSTANCE.get_manga_listing(listing, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	INSTANCE.get_manga_details(id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	INSTANCE.get_chapter_list(id)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	INSTANCE.get_page_list(manga_id, chapter_id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	INSTANCE.modify_image_request(request);
}
