#![no_std]
use aidoku::{
	error::Result, prelude::*, std::net::Request, std::net::HttpMethod, std::String, std::Vec, Chapter, DeepLink, Filter,
	Listing, Manga, MangaPageResult, Page,
};

use mangastream_template::template::MangaStreamSource;

fn get_instance() -> MangaStreamSource {
	MangaStreamSource {
		base_url: String::from("https://rizzcomic.com"),
		traverse_pathname: "series",
		chapter_date_format: "dd MMM yyyy",
		alt_pages: true,
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
	let url = format!("{}/chapter/{}", &get_instance().base_url, id);
	let mut pages: Vec<Page> = Vec::new();
	let html = Request::new(url, HttpMethod::Get).html()?;
		
	for (index, page) in html.select(".rdminimal > img").array().enumerate() {
		let page = page.as_node()?;
		let url = page.attr("src").read();

		pages.push(Page {
			index: index as i32,
			url,
			..Default::default()
		});
	}
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
