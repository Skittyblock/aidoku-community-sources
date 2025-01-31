#![no_std]
use aidoku::{
	error::Result, prelude::*, std::net::Request, std::String, std::Vec, Chapter, DeepLink, Filter,
	Listing, Manga, MangaPageResult, Page,
};

use mangastream_template::template::MangaStreamSource;

fn get_instance() -> MangaStreamSource {
	MangaStreamSource {
		base_url: String::from("https://rizzfables.com"),
		traverse_pathname: "series",
		status_options: ["ongoing", "completed", "hiatus", "cancelled", "dropped"],
		manga_details_description: ".entry-content-single > script",
		manga_details_status: ".bs-status",
		chapter_date_format: "dd MMM yyyy",
		..Default::default()
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	get_instance().parse_manga_list(filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	get_instance().parse_manga_listing(get_instance().base_url, listing.name, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let mut manga = get_instance().parse_manga_details(id)?;

	// the description is stored in a script tag, so we need to extract it
	let content = manga.description;

	let start_pattern = r#"var description = ""#;
	let end_pattern = r#"" ;"#;

	let new_description = if let Some(start_index) = content.find(start_pattern) {
		let start_index = start_index + start_pattern.len();
		if let Some(end_index) = content[start_index..].find(end_pattern) {
			let end_index = start_index + end_index;
			Some(&content[start_index..end_index])
		} else {
			None
		}
	} else {
		None
	};

	manga.description = new_description
		.unwrap_or_default()
		.replace("\\u201d", "\"")
		.replace("\\u201c", "\"")
		.replace("\\u2014", "—")
		.replace("\\u2019", "'")
		.replace("\\u2026", "…")
		.replace("\\r\\n", "\n")
		.replace("\\n", "\n")
		.replace("\\\"", "\"");

	Ok(manga)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	get_instance().parse_chapter_list(id)
}

#[get_page_list]
fn get_page_list(_manga_id: String, id: String) -> Result<Vec<Page>> {
	get_instance().parse_page_list(format!("chapter/{id}"))
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	get_instance().modify_image_request(request)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(url)
}
