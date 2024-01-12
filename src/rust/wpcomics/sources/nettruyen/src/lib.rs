#![no_std]
pub mod helper;
use crate::helper::*;
use aidoku::{
	error::Result,
	prelude::*,
	std::net::Request,
	std::String,
	std::{defaults::defaults_get, Vec},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, MangaViewer, Page,
};
use wpcomics_template::{helper::urlencode, template, template::WPComicsSource};

fn get_instance() -> WPComicsSource {
	WPComicsSource {
		base_url: String::from("https://www.nettruyenclub.com"),
		next_page: "li.active + li > a[title*=\"kết quả\"]",
		viewer: MangaViewer::Rtl,
		listing_mapping: |listing| {
			String::from(match listing.as_str() {
				"Truyện con gái" => "truyen-con-gai",
				"Truyện con trai" => "truyen-con-trai",
				"Hot" => "hot",
				_ => "",
			})
		},
		status_mapping: status_map,
		time_converter: convert_time,
		page_url_transformer: |url| {
			let mut server_two = String::from("https://images2-focus-opensocial.googleusercontent.com/gadgets/proxy?container=focus&gadget=a&no_expand=1&resize_h=0&rewriteMime=image%2F*&url=");
			if let Ok(server_selection) = defaults_get("serverSelection") {
				if let Ok(2) = server_selection.as_int() {
					server_two.push_str(&urlencode(url));
					server_two
				} else {
					url
				}
			} else {
				url
			}
		},
		..Default::default()
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut included_tags: Vec<String> = Vec::new();
	let mut excluded_tags: Vec<String> = Vec::new();
	let mut title: String = String::new();
	let mut sort_by: i32 = 0;
	let mut gender: i32 = -1;
	let mut completed: i32 = -1;
	let mut chapter_count: i32 = 0;
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				title = urlencode(filter.value.as_string()?.read());
			}
			FilterType::Genre => match filter.value.as_int().unwrap_or(-1) {
				0 => excluded_tags.push(get_tag_id(String::from(&filter.name))),
				1 => included_tags.push(get_tag_id(String::from(&filter.name))),
				_ => continue,
			},
			_ => {
				match filter.name.as_str() {
					"Tình trạng" => {
						completed = filter.value.as_int().unwrap_or(-1) as i32;
						if completed == 0 {
							completed = -1;
						}
					}
					"Số lượng chapter" => {
						chapter_count = match filter.value.as_int().unwrap_or(0) {
							0 => 1,
							1 => 50,
							2 => 100,
							3 => 200,
							4 => 300,
							5 => 400,
							6 => 500,
							_ => 1,
						};
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
					"Dành cho" => {
						gender = filter.value.as_int().unwrap_or(-1) as i32;
						if gender == 0 {
							gender = -1;
						}
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
		included_tags,
		excluded_tags,
		sort_by,
		gender,
		completed,
		chapter_count,
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
		String::from("https://www.nettruyenclub.com"),
		String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.0.4951.54 Safari/537.36 Edg/101.0.1210.39"),
		request,
	)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(url)
}
