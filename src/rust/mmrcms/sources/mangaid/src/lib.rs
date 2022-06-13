#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{defaults::defaults_get, String, Vec},
	Chapter, DeepLink, Filter, Manga, MangaPageResult, Page,
};
use lazy_static::lazy_static;
use mmrcms_template::template::MMRCMSSource;

lazy_static! {
	static ref INSTANCE: MMRCMSSource = MMRCMSSource {
		base_url: "https://mangaid.click",
		detail_description: "Description",
		category_mapper: |idx| {
			if idx == 0 {
				String::new()
			} else if (1..=16).contains(&idx) {
				format!("{}", idx)
			} else {
				format!("{}", idx + 1)
			}
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
	let cdn = defaults_get("useCDN").as_string()?.read();
	INSTANCE.get_page_list(format!("{id}{cdn}"))
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	INSTANCE.handle_url(url)
}
