#![no_std]
use aidoku::{
	error::Result, prelude::*, std::String, std::Vec, Chapter, DeepLink, Filter, Listing, Manga,
	MangaPageResult, Page,
};

use madara_template::helper;
use madara_template::template;

fn get_data() -> template::MadaraSiteData {
	let lang_code = helper::get_lang_code();
	let base_url;
	let source_path;

	match lang_code.as_deref() {
		Some("es") => {
			base_url = String::from("https://es.leviatanscans.com");
			source_path = String::from("manga");
		}
		// Default to English
		_ => {
			base_url = String::from("https://en.leviatanscans.com");
			source_path = String::from("tkl/manga");
		}
	}

	let data: template::MadaraSiteData = template::MadaraSiteData {
		base_url,
		source_path,
		chapter_selector: String::from("li.wp-manga-chapter.free-chap"),
		description_selector: String::from(
			"div.summary_content div.post-content div.post-content_item div p",
		),
		alt_ajax: true,
		..Default::default()
	};
	data
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	template::get_manga_list(filters, page, get_data())
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	template::get_manga_listing(get_data(), listing, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	template::get_manga_details(id, get_data())
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	template::get_chapter_list(id, get_data())
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	template::get_page_list(id, get_data())
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	template::handle_url(url, get_data())
}
