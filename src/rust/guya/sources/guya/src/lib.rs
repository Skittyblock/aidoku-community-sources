#![no_std]
use aidoku::{
	error::Result, prelude::*, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Chapter, DeepLink, Filter, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

use guya_template::template;

fn data() -> template::GuyaSiteData {
	template::GuyaSiteData {
		base_url: String::from("https://guya.cubari.moe"),
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	template::get_manga_list(data(), filters, page)
}

#[get_manga_listing]
pub fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	todo!()
}

#[get_manga_details]
pub fn get_manga_details(id: String) -> Result<Manga> {
	todo!()
}

#[get_chapter_list]
pub fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	todo!()
}

#[get_page_list]
pub fn get_page_list(id: String, _: String) -> Result<Vec<Page>> {
	todo!()
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	todo!()
}
