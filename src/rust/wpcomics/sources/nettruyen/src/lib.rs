#![no_std]
pub mod helper;
use crate::helper::*;
use aidoku::{
	error::Result, prelude::*, std::net::Request, std::String, std::Vec, Chapter, DeepLink, Filter,
	FilterType, Listing, Manga, MangaPageResult, MangaViewer, Page,
};
use wpcomics_template::{helper::urlencode, template, template::WPComicsSource};

const BASE_URL: &str = "https://nettruyenvia.com";

fn get_instance() -> WPComicsSource {
	WPComicsSource {
		base_url: String::from(BASE_URL),
		next_page: "li.active + li > a[title*=\"kết quả\"]",
		viewer: MangaViewer::Rtl,
		listing_mapping: |listing| {
			String::from(match listing.as_str() {
				"Truyện con gái" => "truyen-tranh-con-gai",
				"Truyện con trai" => "truyen-tranh-con-trai",
				"Hot" => "truyen-tranh-hot",
				_ => "",
			})
		},
		status_mapping: status_map,
		time_converter: convert_time,
		manga_viewer_page_attr: "data-src",
		..Default::default()
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut category: Option<String> = None;
	let mut title: String = String::new();
	let mut sort_by: i32 = 0;
	let mut completed: i32 = -1;
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				title = urlencode(filter.value.as_string()?.read());
			}
			FilterType::Genre => {
				category = Some(filter.value.as_string()?.read());
			}
			_ => {
				match filter.name.as_str() {
					"Tình trạng" => {
						completed = filter.value.as_int().unwrap_or(-1) as i32;
						if completed == 0 {
							completed = -1;
						}
					}
					"Sắp xếp theo" => {
						sort_by = match filter.value.as_int().unwrap_or(0) {
							0 => 0,  // new chapters
							1 => 15, // new mangas
							2 => 10, // most watched
							3 => 11, // most watched this month
							4 => 12, // most watched this week
							5 => 13, // most watched today
							6 => 20, // most followed
							7 => 25, // most commented
							8 => 30, // most chapters
							9 => 5,  // alphabetical
							_ => 0,
						};
					}
					_ => continue,
				}
			}
		}
	}
	let instance = get_instance();
	instance.get_manga_list(get_search_url(
		instance.base_url.clone(),
		title,
		page,
		category,
		sort_by,
		completed,
	))
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	get_instance().get_manga_listing(listing, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	get_instance().get_manga_details(id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	get_instance().get_chapter_list(id)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	get_instance().get_page_list(chapter_id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	template::modify_image_request(
		String::from(BASE_URL),
		String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.0.4951.54 Safari/537.36 Edg/101.0.1210.39"),
		request,
	)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(url)
}
