#![no_std]
#![feature(let_chains)]
extern crate alloc;
mod helper;
pub mod template;

#[macro_export]
macro_rules! source {
	($a:ident) => {
		#[aidoku::prelude::get_manga_list]
		fn get_manga_list(
			filters: aidoku::std::Vec<Filter>,
			page: i32,
		) -> aidoku::error::Result<MangaPageResult> {
			Ok($a.get_manga_list(filters, page).unwrap())
		}

		#[aidoku::prelude::get_manga_listing]
		fn get_manga_listing(
			listing: aidoku::Listing,
			page: i32,
		) -> aidoku::error::Result<MangaPageResult> {
			Ok($a.get_manga_listing(listing, page).unwrap())
		}

		#[aidoku::prelude::get_manga_details]
		fn get_manga_details(id: aidoku::std::String) -> aidoku::error::Result<aidoku::Manga> {
			Ok($a.get_manga_details(id).unwrap())
		}

		#[aidoku::prelude::get_chapter_list]
		fn get_chapter_list(
			id: aidoku::std::String,
		) -> aidoku::error::Result<aidoku::std::Vec<aidoku::Chapter>> {
			Ok($a.get_chapter_list(id).unwrap())
		}

		#[aidoku::prelude::get_page_list]
		fn get_page_list(
			manga_id: aidoku::std::String,
			id: aidoku::std::String,
		) -> aidoku::error::Result<aidoku::std::Vec<aidoku::Page>> {
			Ok($a.get_page_list(manga_id, id).unwrap())
		}

		#[aidoku::prelude::modify_image_request]
		fn modify_image_request(request: aidoku::std::net::Request) {
			$a.modify_image_request(request)
		}

		#[aidoku::prelude::handle_url]
		fn handle_url(url: aidoku::std::String) -> aidoku::error::Result<aidoku::DeepLink> {
			Ok($a.handle_url(url).unwrap())
		}

		#[aidoku::prelude::handle_notification]
		fn handle_notification(notification: aidoku::std::String) {
			$a.handle_notification(notification)
		}
	};
}
