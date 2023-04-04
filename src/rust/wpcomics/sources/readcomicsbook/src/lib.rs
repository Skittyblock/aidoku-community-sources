#![no_std]
mod helper;
mod parser;
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::HttpMethod, net::Request, String, Vec},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, Page,
};
use helper::{get_search_url, listing_mapping};
use parser::parse_comic;
use wpcomics_template::{
	helper::{get_tag_id, urlencode},
	template::{self, *},
};

fn get_instance() -> WPComicsSource {
	WPComicsSource {
		base_url: String::from("https://readcomicsbook.com"),
		listing_mapping,

		manga_cell: "li[itemtype=\"https://schema.org/Book\"]",
		manga_cell_title: "div.manga-info > h3 > a",
		manga_cell_url: "div.manga-info > h3 > a",
		manga_cell_image: "div.manga-thumb > a > img",

		manga_details_title: "div.headline > h2[itemprop=name]",
		manga_details_cover: "div.manga-thumb > img",
		manga_details_author: "div.mt-author",
		manga_details_author_transformer: |title| title.replace("Author(s): ", ""),
		manga_details_description: "div.summary-content",
		manga_details_tags: "div.meta-data.view + div.meta-data",
		manga_details_status: "div.meta-data:contains(Status)",
		manga_details_status_transformer: |title| title.replace("Status: ", ""),
		manga_details_chapters: "ul.chapter-list > li",

		chapter_skip_first: true,
		chapter_anchor_selector: "span > a",
		chapter_date_selector: "span.time",

		manga_viewer_page_url_suffix: "/all",

		..Default::default()
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut title: String = String::new();
	let mut genre: String = String::new();
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				title = urlencode(filter.value.as_string()?.read());
			}
			_ => match filter.name.as_str() {
				"Genre" => {
					genre = get_tag_id(filter.value.as_int().unwrap_or(0));
				}
				_ => continue,
			},
		}
	}
	if !title.is_empty() {
		let json = Request::new(
			format!("https://readcomicsbook.com/ajax/search?q={title}").as_str(),
			HttpMethod::Get,
		)
		.json()?
		.as_object()?;
		let result = json.get("data").as_array()?;
		let mut manga_arr: Vec<Manga> = Vec::new();
		for manga in result {
			let manga_obj = manga.as_object()?;
			if let Ok(manga) = parse_comic(String::from("https://readcomicsbook.com"), manga_obj) {
				manga_arr.push(manga);
			}
		}
		Ok(MangaPageResult {
			manga: manga_arr,
			has_more: false,
		})
	} else {
		get_instance().get_manga_list(get_search_url(
			String::from("https://readcomicsbook.com"),
			genre,
			page,
		))
	}
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
		String::from("https://readcomicsbook.com"),
		String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.0.4951.54 Safari/537.36 Edg/101.0.1210.39"),
		request,
	)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(url)
}
