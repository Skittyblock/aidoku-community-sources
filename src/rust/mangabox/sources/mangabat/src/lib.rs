#![no_std]
use aidoku::{
	error::Result, prelude::*, std::net::Request, std::String, std::Vec, Chapter, DeepLink, Filter,
	Listing, Manga, MangaPageResult, Page,
};
use mangabox_template::template;

const BASE_URL: &str = "https://www.mangabats.com";
const ITEM_SELECTOR: &str = ".panel_story_list .story_item, .list-truyen-item-wrap";
const GENRES: [&str; 41] = [
	"all",
	"action",
	"adult",
	"adventure",
	"comedy",
	"cooking",
	"doujinshi",
	"drama",
	"ecchi",
	"fantasy",
	"gender-bender",
	"harem",
	"historical",
	"horror",
	"isekai",
	"josei",
	"manhua",
	"manhwa",
	"martial-arts",
	"mature",
	"mecha",
	"medical",
	"mystery",
	"one-shot",
	"psychological",
	"romance",
	"school-life",
	"sci-fi",
	"seinen",
	"shoujo",
	"shoujo-ai",
	"shounen",
	"shounen-ai",
	"slice-of-life",
	"smut",
	"sports",
	"supernatural",
	"tragedy",
	"webtoons",
	"yaoi",
	"yuri",
];

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	template::get_manga_list(
		BASE_URL,
		ITEM_SELECTOR,
		filters,
		page,
		false,
		Some("/search/story"),
		Some(&GENRES),
	)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	template::get_manga_listing(
		BASE_URL,
		ITEM_SELECTOR,
		listing,
		page,
		false,
		Some("/search/story"),
		Some(&GENRES),
	)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	template::get_manga_details(id, BASE_URL, Some("div#contentBox"))
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	template::get_chapter_list(id, BASE_URL, "MMM-dd-yyyy HH:mm")
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	template::get_page_list(chapter_id, BASE_URL)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	template::modify_image_request(&format!("{BASE_URL}/"), request)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	template::handle_url(url, BASE_URL)
}
