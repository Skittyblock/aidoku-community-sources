#![no_std]
#![feature(let_chains)]
use aidoku::{
	error::Result,
	prelude::*,
	std::{defaults::defaults_get, String, Vec},
	Chapter, DeepLink, Filter, Manga, MangaPageResult, Page,
};
use lazy_static::lazy_static;
use mmrcms_template::template::MMRCMSSource;

lazy_static! {
	static ref INSTANCE: MMRCMSSource<'static> = MMRCMSSource {
		base_url: "https://mangaid.click",
		category_mapper: |idx| {
			if idx == 0 {
				String::new()
			} else if (1..=16).contains(&idx) {
				String::from(itoa::Buffer::new().format(idx))
			} else {
				String::from(itoa::Buffer::new().format(idx + 1))
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
fn get_page_list(manga_id: String, id: String) -> Result<Vec<Page>> {
	let cdn = if let Ok(default) = defaults_get("useCDN")
				 && let Ok(cdn) = default.as_string().map(|v| v.read()) {
		cdn
	} else {
		String::from("?cdn=off")
	};
	INSTANCE.get_page_list(manga_id, format!("{id}{cdn}"))
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	INSTANCE.handle_url(url)
}
