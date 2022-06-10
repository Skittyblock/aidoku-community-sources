#![no_std]
use aidoku::{
	error::Result, prelude::*, std::String, std::Vec, Chapter, DeepLink, Filter, Listing, Manga,
	MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page,
};

use madara_template::template;

fn get_data() -> template::MadaraSiteData {
	let data: template::MadaraSiteData = template::MadaraSiteData {
		base_url: String::from("https://fecomic.com"),
		lang: String::from("vi"),
		source_path: String::from("comic"),
		genre_selector: String::from("div.genres > a"),
		alt_ajax: true,
		viewer: |_, categories| {
			for category in categories {
				match category.as_str() {
					"Manhwa" | "Manhua" | "Webtoon" => return MangaViewer::Scroll,
					_ => continue,
				}
			}
			MangaViewer::Rtl
		},
		status: |html| {
			let status = html.select("div.post-status").text().read();
			match status.trim() {
				"Hoàn" => MangaStatus::Completed,
				"Đang dịch" => MangaStatus::Ongoing,
				"Ngừng dịch" => MangaStatus::Cancelled,
				"Ngang raw" => MangaStatus::Hiatus,
				_ => MangaStatus::Unknown,
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
						"smut" | "mature" | "adult" | "truyện 18+" => {
							return MangaContentRating::Nsfw
						}
						"ecchi" | "16+" => nsfw = MangaContentRating::Suggestive,
						_ => continue,
					}
				}
				nsfw
			}
		},
		status_filter_ongoing: String::from("Đang tiến hành"),
		status_filter_completed: String::from("Đã hoàn thành"),
		status_filter_cancelled: String::from("Đã bị huỷ/Ngừng dịch"),
		status_filter_on_hold: String::from("Tạm ngưng/Ngang raw"),
		adult_string: String::from("Truyện 18+"),
		genre_condition: String::from("Điều kiện lọc thể loại"),
		trending: String::from("Truyện hot"),
		popular: String::from("Phổ biến"),
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
