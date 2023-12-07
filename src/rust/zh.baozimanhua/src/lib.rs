#![no_std]
mod url;

use aidoku::{
	error::Result,
	prelude::{get_manga_list, println},
	std::Vec,
	Filter, MangaPageResult,
};
use url::Url;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let filters_url = Url::from((filters, page));
	println!("{}", filters_url);
	todo!()
}

// #[get_manga_listing]
// fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult>
// { 	todo!()
// }

// #[get_manga_details]
// fn get_manga_details(id: String) -> Result<Manga> {
// 	todo!()
// }

// #[get_chapter_list]
// fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
// 	todo!()
// }

// #[get_page_list]
// fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
// 	todo!()
// }

// #[modify_image_request]
// fn modify_image_request(request: Request) {
// 	todo!()
// }

// #[handle_url]
// fn handle_url(url: String) -> Result<DeepLink> {
// 	todo!()
// }
