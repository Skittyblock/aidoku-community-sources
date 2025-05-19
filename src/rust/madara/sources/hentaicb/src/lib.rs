#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::String,
	std::{net::Request, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

use madara_template::template;

fn get_data() -> template::MadaraSiteData {
	let data: template::MadaraSiteData = template::MadaraSiteData {
		base_url: String::from("https://hentaicube.xyz"),
		lang: String::from("vi"),
		source_path: String::from("read"),
		image_selector: String::from(".reading-content .doc-truyen > img"),
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
			let status = html
				.select("div.post-content_item:contains(Tình trạng) div.summary-content")
				.text()
				.read();
			match status.to_lowercase().trim() {
				"hoàn thành" => MangaStatus::Completed,
				"đang tiến hành" => MangaStatus::Ongoing,
				"đã huỷ" => MangaStatus::Cancelled,
				"tạm ngưng" => MangaStatus::Hiatus,
				_ => MangaStatus::Unknown,
			}
		},
		nsfw: |_, _| MangaContentRating::Nsfw,
		status_filter_ongoing: String::from("Đang tiến hành"),
		status_filter_completed: String::from("Hoàn thành"),
		status_filter_cancelled: String::from("Đã huỷ"),
		status_filter_on_hold: String::from("Tạm ngưng"),
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

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.header("Referer", "https://hentaicube.xyz/");
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	template::get_page_list(chapter_id, get_data())
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	template::handle_url(url, get_data())
}
