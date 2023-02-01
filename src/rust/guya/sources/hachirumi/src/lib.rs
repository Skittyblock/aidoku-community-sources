#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::net::HttpMethod,
	std::net::Request,
	std::String,
	std::{ObjectRef, Vec},
	Chapter, DeepLink, Filter, Manga, MangaContentRating, MangaPageResult, Page,
};
use alloc::string::ToString;
extern crate alloc;
use guya_template::template;

fn data() -> template::GuyaSiteData {
	template::GuyaSiteData {
		base_url: String::from("https://hachirumi.com"),
		..Default::default()
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	template::get_manga_list(data(), filters, page)
}

#[get_manga_details]
pub fn get_manga_details(slug: String) -> Result<Manga> {
	let url = format!("{}/read/series/{}/", &data().base_url, slug);
	let request = Request::new(url, HttpMethod::Get).header("User-Agent", "Aidoku");
	let html = request.html()?;
	let nsfw_modal = html.select("script");
	if nsfw_modal
		.html()
		.read()
		.contains("$('#nfsw-modal').modal({backdrop: 'static', keyboard: false});")
	{
		return template::get_manga_details(data(), slug, MangaContentRating::Nsfw);
	}
	template::get_manga_details(data(), slug, MangaContentRating::Safe)
}

#[get_chapter_list]
pub fn get_chapter_list(slug: String) -> Result<Vec<Chapter>> {
	template::get_chapter_list(data(), slug)
}

/// # Safety
///
/// I have no clue why this is unsafe tbh, took this from aidoku-rs as I needed
/// the full chapter obj. Clippy making me put a safety comment here.
#[no_mangle]
#[export_name = "get_page_list"]
pub unsafe extern "C" fn __wasm_get_page_list(rid: i32) -> i32 {
	let obj = aidoku::std::ObjectRef(aidoku::std::ValueRef::new(rid));
	let resp: Result<Vec<Page>> = get_page_list(obj);
	match resp {
		Ok(resp) => {
			let mut arr = aidoku::std::ArrayRef::new();
			for item in resp {
				let rid = item.create();
				arr.insert(aidoku::std::ValueRef::new(rid));
			}
			let rid = arr.0 .0;
			core::mem::forget(arr.0);
			rid
		}
		Err(_) => -1,
	}
}

pub fn get_page_list(chapter: ObjectRef) -> Result<Vec<Page>> {
	template::get_page_list(data(), chapter)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let parts = url.split('/').collect::<Vec<&str>>();
	let slug = parts[5].to_string();
	let nsfw_url = format!("{}/read/series/{}/", &data().base_url, slug);
	let request = Request::new(nsfw_url, HttpMethod::Get).header("User-Agent", "Aidoku");
	let html = request.html()?;
	let nsfw_modal = html.select("script");
	if nsfw_modal
		.html()
		.read()
		.contains("$('#nfsw-modal').modal({backdrop: 'static', keyboard: false});")
	{
		return template::handle_url(data(), url, MangaContentRating::Nsfw);
	}
	template::handle_url(data(), url, MangaContentRating::Safe)
}
