#![no_std]
use aidoku::{
	error::Result, prelude::*, std::String, std::Vec, Chapter, DeepLink, Filter, Listing, Manga,
	MangaContentRating, MangaPageResult, MangaViewer, Page,
};

use madara_template::template;

fn get_data() -> template::MadaraSiteData {
	let data: template::MadaraSiteData = template::MadaraSiteData {
		base_url: String::from("https://lilymanga.net"),
		source_path: String::from("ys"),
		viewer: |html, _| {
			let temp = html
				.select("div.post-content_item:contains(Type) div.summary-content")
				.text()
				.read();
			match temp.as_str() {
				"Manhwa" | "Manhua" => MangaViewer::Scroll,
				_ => MangaViewer::Rtl,
			}
		},
		nsfw: |html, categories| {
			if !html
				.select(".manga-title-badges.adult")
				.text()
				.read()
				.is_empty()
			{
				MangaContentRating::Nsfw
			} else {
				let mut nsfw = MangaContentRating::Safe;
				for category in categories {
					match category.to_lowercase().as_str() {
						"smut" | "mature" | "adult" | "hentai" => return MangaContentRating::Nsfw,
						"ecchi" => nsfw = MangaContentRating::Suggestive,
						_ => continue,
					}
				}
				nsfw
			}
		},
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
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	template::get_page_list(chapter_id, get_data())
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	template::handle_url(url, get_data())
}
