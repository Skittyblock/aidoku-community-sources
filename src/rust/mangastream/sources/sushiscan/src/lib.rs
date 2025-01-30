#![no_std]
use aidoku::{
	error::Result, prelude::*, std::net::Request, std::String, std::Vec, Chapter, DeepLink, Filter,
	Listing, Manga, MangaPageResult, Page,
};

use mangastream_template::template::{MangaStreamSource, USER_AGENT};

const BASE_URL: &str = "https://sushiscan.net";

fn get_instance() -> MangaStreamSource {
	MangaStreamSource {
		base_url: String::from(BASE_URL),
		traverse_pathname: "catalogue",
		listing: ["Dernières", "Populaire", "Nouveau"],
		status_options: ["En Cours", "Terminé", "En Pause", "Abandonné", ""],
		manga_details_categories: ".seriestugenre a",
		manga_details_author: ".infotable tr:contains(Auteur) td:last-child",
		manga_details_status: ".infotable tr:contains(Statut) td:last-child",
		manga_details_type: ".infotable tr:contains(Type) td:last-child",
		language: "fr",
		locale: "fr-FR",
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
	get_instance().parse_manga_details(id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	get_instance().parse_chapter_list(id)
}

#[get_page_list]
fn get_page_list(_manga_id: String, id: String) -> Result<Vec<Page>> {
	let html = Request::get(format!("{BASE_URL}/{id}"))
		.header("Referer", BASE_URL)
		.header("User-Agent", USER_AGENT)
		.html()?;

	let content = html.select(".readercontent").html().read(); // contains the script tag we're targeting

	let start_pattern = r#"ts_reader.run("#;
	let end_pattern = r#");"#;

	let json = if let Some(start_index) = content.find(start_pattern) {
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

	let Some(json) = json else {
		return Err(aidoku::error::AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		});
	};

	let json = aidoku::std::json::parse(json)?.as_object()?;

	let mut sources = json.get("sources").as_array()?;

	let pages = sources
		.next()
		.ok_or(aidoku::error::AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		})?
		.as_object()?
		.get("images")
		.as_array()?
		.enumerate()
		.filter_map(|(idx, url)| {
			Some(Page {
				index: idx as i32,
				url: url.as_string().ok()?.read().replace("http://", "https://"),
				..Default::default()
			})
		})
		.collect();

	Ok(pages)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	get_instance().modify_image_request(request)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(url)
}
