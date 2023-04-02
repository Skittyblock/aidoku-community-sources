#![no_std]
mod helper;
use crate::helper::*;
use aidoku::{
	error::Result,
	prelude::*,
	std::{defaults::defaults_get, net::Request, String, StringRef, Vec},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaPageResult, MangaStatus, MangaViewer, Page,
};
use wpcomics_template::{helper::urlencode, template::WPComicsSource};

fn get_instance() -> WPComicsSource {
	let base_url = defaults_get("sourceURL")
		.expect("sourceURL is not set")
		.as_string()
		.unwrap_or_else(|_| StringRef::from(""))
		.read();
	WPComicsSource {
		base_url,
		viewer: MangaViewer::Rtl,
		time_converter: convert_time,
		status_mapping: |arg1| match arg1.as_str() {
			"Đang tiến hành" => MangaStatus::Ongoing,
			"Đã hoàn thành" => MangaStatus::Completed,
			_ => MangaStatus::Unknown,
		},
		next_page: "li > a:contains(Cuối »)",
		manga_cell_image: "div.image img",
		..Default::default()
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut title = String::new();
	let mut status: i64 = -1;
	let mut sort: i64 = -1;
	let mut genre = String::new();
	for filter in filters {
		match filter.kind {
			FilterType::Title => title = urlencode(filter.value.as_string()?.read()),
			FilterType::Select => {
				let value = filter.value.as_int().unwrap_or(-1);
				if value < 0 {
					continue;
				}
				match filter.name.as_str() {
					"Tình trạng" => status = value,
					"Sắp xếp theo" => {
						sort = match value {
							0 => 0,
							1 => 15,
							2 => 10,
							3 => 11,
							4 => 12,
							5 => 13,
							_ => -1,
						}
					}
					"Thể loại" => genre = genre_mapping(value),
					_ => continue,
				}
			}
			_ => continue,
		}
	}
	let mut url = format!(
		"{}/tim-truyen?genre={}&keyword={}&page={}",
		defaults_get("sourceURL")
			.expect("sourceURL is not set")
			.as_string()
			.unwrap_or_else(|_| StringRef::from(""))
			.read(),
		genre,
		title,
		page,
	);
	if status > 0 {
		url.push_str("&status=");
		url.push_str(format!("{}", status).as_str());
	}
	if sort > 0 {
		url.push_str("&sort=");
		url.push_str(format!("{}", sort).as_str());
	}
	get_instance().get_manga_list(url)
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
	get_instance().modify_image_request(request)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(url)
}
