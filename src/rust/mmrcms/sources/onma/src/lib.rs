#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, Manga, MangaContentRating, MangaPageResult, MangaViewer, Page,
};
use lazy_static::lazy_static;
use mmrcms_template::template::MMRCMSSource;

lazy_static! {
	static ref INSTANCE: MMRCMSSource = MMRCMSSource {
		base_url: "https://onma.me",
		lang: "ar",

		category: "الفئة",
		tags: "العلامات",

		details_title_selector: "div.panel-heading",
		detail_description: "نبذة عن المانجا",
		detail_status_ongoing: "مستمرة",
		detail_status_complete: "مكتملة",

		category_parser: |_, categories| {
			let mut nsfw = MangaContentRating::Safe;
			let mut viewer = MangaViewer::Rtl;
			for category in categories {
				match category.as_str() {
					// "Sexual perversion" | "Mature"
					"انحراف جنسي" | "ناضج" => nsfw = MangaContentRating::Nsfw,
					// Webtoon
					"ويب تون" => viewer = MangaViewer::Scroll,
					_ => continue,
				}
			}
			(nsfw, viewer)
		},
		..Default::default()
	};
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
fn get_page_list(id: String) -> Result<Vec<Page>> {
	INSTANCE.get_page_list(id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	INSTANCE.modify_image_request(request)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	INSTANCE.handle_url(url)
}
