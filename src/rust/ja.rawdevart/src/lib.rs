#![no_std]
use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	helpers::{substring::Substring, uri::QueryParameters},
	prelude::*,
	std::{
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, Page,
};

extern crate alloc;
use alloc::string::ToString;

mod parser;

const BASE_URL: &str = "https://rawdevart.art";
const PAGE_URL: &str = "https://s1.rawuwu.com";

// generate genre filters:
// names: copy([...$0.querySelectorAll("#m-genres option")].map((e) => `"${e.textContent}"`).join(", "))
// values: copy([...$0.querySelectorAll("#m-genres option")].map((e) => `"${e.value.split("/").slice(1, 3).join("/")}"`).join(", "))
const GENRES: &[&str] = &[
	"genres", // replace "all" with "genres"
	"genre/85",
	"genre/139",
	"genre/86",
	"genre/149",
	"genre/140",
	"genre/87",
	"genre/134",
	"genre/114",
	"genre/88",
	"genre/150",
	"genre/89",
	"genre/152",
	"genre/155",
	"genre/111",
	"genre/90",
	"genre/115",
	"genre/127",
	"genre/144",
	"genre/130",
	"genre/91",
	"genre/148",
	"genre/151",
	"genre/128",
	"genre/125",
	"genre/126",
	"genre/112",
	"genre/143",
	"genre/132",
	"genre/141",
	"genre/121",
	"genre/156",
	"genre/142",
	"genre/157",
	"genre/119",
	"genre/106",
	"genre/108",
	"genre/122",
	"genre/146",
	"genre/107",
	"genre/154",
	"genre/120",
	"genre/131",
	"genre/118",
	"genre/109",
	"genre/92",
	"genre/123",
	"genre/124",
	"genre/93",
	"genre/135",
	"genre/138",
	"genre/147",
	"genre/153",
	"genre/116",
	"genre/161",
	"genre/110",
];
const STATUSES: &[&str] = &["", "ongoing", "completed"];
const SORTS: &[&str] = &["", "most_viewed", "most_viewed_today"];

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut params = QueryParameters::new();
	let mut path: Option<String> = None;

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(title) = filter.value.as_string() {
					// reset other filters, as title is the only filter we use if it's active
					params = QueryParameters::new();
					params.push("query", Some(&title.read()));
					path = Some(String::from("search"));
					break;
				}
			}
			FilterType::Select => {
				let value = match filter.value.as_int() {
					Ok(value) => value,
					Err(_) => continue,
				} as usize;
				match filter.name.as_str() {
					"Genre" => path = Some(String::from(GENRES[value])),
					"Status" => params.push("status", Some(STATUSES[value])),
					_ => continue,
				}
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int().unwrap_or(0) as usize;
				params.push("sort", Some(SORTS[index]));
			}
			_ => continue,
		}
	}

	params.push("page", Some(&page.to_string()));

	let path = match path {
		Some(path) => path,
		None => "genres".to_string(),
	};

	let url = format!("{BASE_URL}/spa/{path}?{params}");
	parse_manga_list(url)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let url = match listing.name.as_str() {
		"Popular" => format!("{BASE_URL}/spa/genres?page={page}&sort=most_viewed"),
		_ => format!("{BASE_URL}/spa/genres?page={page}"),
	};
	parse_manga_list(url)
}

fn parse_manga_list(url: String) -> Result<MangaPageResult> {
	let json = Request::new(url, HttpMethod::Get).json()?.as_object()?;

	let manga: Vec<Manga> = json
		.get("manga_list")
		.as_array()?
		.filter_map(|manga| match manga.as_object() {
			Ok(obj) => parser::parse_basic_manga(&obj).ok(),
			Err(_) => None,
		})
		.collect::<Vec<_>>();

	let has_more = json
		.get("pagi")
		.as_object()
		.and_then(|obj| obj.get("button").as_object())
		.and_then(|obj| obj.get("next").as_int())
		.is_ok_and(|n| n != 0);

	Ok(MangaPageResult { manga, has_more })
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{BASE_URL}/spa/manga/{id}");
	let json = Request::new(url, HttpMethod::Get).json()?.as_object()?;
	parser::parse_manga_details(&json)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("{BASE_URL}/spa/manga/{id}");
	let json = Request::new(url, HttpMethod::Get).json()?.as_object()?;
	parser::parse_chapters(&json)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{BASE_URL}/spa/manga/{manga_id}/{chapter_id}");
	let json = Request::new(url, HttpMethod::Get).json()?.as_object()?;
	parser::parse_pages(&json)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.header("Referer", &format!("{BASE_URL}/"));
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	// manga: https://rawdevart.art/yasashii-kazoku-to-takusan-no-mofumofu-ni-kakomarete-c8872
	// chapter: https://rawdevart.art/read/yasashii-kazoku-to-takusan-no-mofumofu-ni-kakomarete-c8872/chapter-35.2

	let split = url.split('/').collect::<Vec<&str>>();
	if split.len() >= 3 {
		let manga_id = {
			let split_idx = if split[3] == "read" { 4 } else { 3 };
			let start_idx = split[split_idx].rfind('c').map_or(0, |c| c + 1);
			&split[split_idx][start_idx..]
		};
		let chapter_id = if split[3] == "read" && split.len() > 4 {
			split[5].substring_after("chapter-")
		} else {
			None
		};

		let manga = get_manga_details(manga_id.to_string()).ok();
		let chapter = chapter_id.map(|id| Chapter {
			id: id.to_string(),
			..Default::default()
		});

		Ok(DeepLink { manga, chapter })
	} else {
		Err(AidokuError {
			reason: AidokuErrorKind::Unimplemented,
		})
	}
}
