#![no_std]
#![feature(stmt_expr_attributes)]
#![feature(let_chains)]
pub mod helper;
pub mod template;

#[macro_export]
macro_rules! mmrcms {
	($e:expr) => {
		use aidoku::{
			error::Result,
			prelude::*,
			std::{net::Request, String, Vec},
			Chapter, DeepLink, Filter, Manga, MangaPageResult, Page,
		};
		use lazy_static::lazy_static;

		lazy_static! {
			static ref INSTANCE: MMRCMSSource<'static> = $e;
		}

		#[get_manga_list]
		fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
			INSTANCE.get_manga_list(filters, page)
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
		fn get_page_list(manga_id: String, id: String) -> Result<Vec<Page>> {
			INSTANCE.get_page_list(manga_id, id)
		}

		#[modify_image_request]
		fn modify_image_request(request: Request) {
			INSTANCE.modify_image_request(request)
		}

		#[handle_url]
		fn handle_url(url: String) -> Result<DeepLink> {
			INSTANCE.handle_url(url)
		}
	};
	() => {};
}
