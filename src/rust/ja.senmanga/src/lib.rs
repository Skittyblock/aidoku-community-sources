#![no_std]
use aidoku::{
	error::Result,
	helpers::uri::{encode_uri, QueryParameters},
	prelude::*,
	std::{net::HttpMethod, net::Request, String, Vec},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult,
	MangaStatus, MangaViewer, Page,
};

extern crate alloc;
use alloc::string::ToString;

const BASE_URL: &str = "https://raw.senmanga.com";

const STATUSES: &[&str] = &["", "Ongoing", "Completed", "Hiatus"];
const TYPES: &[&str] = &["", "Manga", "Manhua", "Manhwa"];
const ORDERS: &[&str] = &["", "A-Z", "Z-A", "Update", "Added", "Popular", "Rating"];

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut params = QueryParameters::new();
	params.push("page", Some(&page.to_string()));

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(title) = filter.value.as_string() {
					params.push("s", Some(&title.read()));
				}
			}
			FilterType::Genre => {
				if filter.value.as_int().unwrap_or(-1) == 1 {
					params.push("genre[]", Some(&filter.name));
				}
			}
			FilterType::Select => {
				let value = match filter.value.as_int() {
					Ok(value) => value,
					Err(_) => continue,
				} as usize;
				match filter.name.as_str() {
					"Status" => params.push("status", Some(STATUSES[value])),
					"Type" => params.push("type", Some(TYPES[value])),
					"Order" => params.push("order", Some(ORDERS[value])),
					_ => continue,
				}
			}
			_ => continue,
		}
	}

	let url = format!("{BASE_URL}/search?{params}");
	parse_manga_list(url)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let url = match listing.name.as_str() {
		"Popular" => format!("{BASE_URL}/directory/popular?page={page}"),
		"Last Update" => format!("{BASE_URL}/directory/last_update?page={page}"),
		"New Series" => format!("{BASE_URL}/directory/new_series?page={page}"),
		_ => String::from(BASE_URL),
	};
	parse_manga_list(url)
}

fn parse_manga_list(url: String) -> Result<MangaPageResult> {
	let html = Request::new(url, HttpMethod::Get).html()?;

	let elements = html.select(".mng");

	let mut manga: Vec<Manga> = Vec::new();

	for element in elements.array() {
		let item = element.as_node().expect("html array not an array of nodes");

		let url = item.select("a").attr("href").read();
		let id = url.strip_prefix(BASE_URL).unwrap_or(&url).to_string();
		let title = item.select(".series-title").text().read();
		let cover = item.select(".cover img").attr("data-src").read();

		manga.push(Manga {
			id,
			cover,
			title,
			url,
			..Default::default()
		})
	}

	let has_more = !html.select("ul.pagination a[rel=next]").array().is_empty();

	Ok(MangaPageResult { manga, has_more })
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{BASE_URL}{id}");
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;

	let cover = html.select(".cover img").attr("src").read();
	let title = html.select("h1.series").text().read();
	let description = html.select(".summary").text().read();

	let info = html.select(".series-desc .info");
	let categories = info
		.select(".item.genre a")
		.array()
		.map(|e| e.as_node().expect("node").text().read())
		.collect::<Vec<String>>();

	let status = match info
		.select(".item:has(strong:contains(Status))")
		.first()
		.own_text()
		.read()
		.as_str()
	{
		"Ongoing" => MangaStatus::Ongoing,
		"Completed" => MangaStatus::Completed,
		"Hiatus" => MangaStatus::Hiatus,
		"Dropped" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	};
	let author = info
		.select(".item:has(strong:contains(Author)) a")
		.text()
		.read();
	let viewer = match info
		.select(".item:has(strong:contains(Type))")
		.first()
		.own_text()
		.read()
		.as_str()
	{
		"Manga" => MangaViewer::Rtl,
		"Manhwa" | "Manhua" => MangaViewer::Scroll,
		_ => MangaViewer::Rtl,
	};
	let nsfw = if categories
		.iter()
		.any(|v| *v == "Ecchi" || *v == "Mature" || *v == "Adult")
	{
		MangaContentRating::Nsfw
	} else {
		MangaContentRating::Safe
	};

	Ok(Manga {
		id,
		cover,
		title,
		author,
		description,
		url,
		categories,
		status,
		nsfw,
		viewer,
		..Default::default()
	})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("{BASE_URL}{id}");
	let html = Request::new(url, HttpMethod::Get).html()?;

	let elements = html.select(".chapter-list li");

	let mut chapters: Vec<Chapter> = Vec::new();

	for element in elements.array() {
		let item = element.as_node().expect("html array not an array of nodes");

		let link = item.select("a");
		let url = link.attr("href").read();
		let id = url.strip_prefix(BASE_URL).unwrap_or(&url).to_string();

		let (title, chapter) = {
			let full_title = link.text().read();
			let mut split_title = full_title.split(' ').skip(1);
			let next = split_title.next();
			// silly chapter number parser that just checks if second word is a number
			let chapter = match next {
				Some(str) => str.parse::<f32>().unwrap_or(-1.0),
				None => -1.0,
			};
			if chapter < 0.0 {
				(full_title, -1.0)
			} else {
				let title = split_title
					.filter(|e| *e != "-")
					.collect::<Vec<&str>>()
					.join(" ");
				(title, chapter)
			}
		};
		let date_updated =
			item.select("time")
				.attr("datetime")
				.as_date("yyyy-MM-dd HH:mm:ss", None, None);

		chapters.push(Chapter {
			id,
			title,
			chapter,
			date_updated,
			url,
			lang: String::from("ja"),
			..Default::default()
		})
	}

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{BASE_URL}{chapter_id}");
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;

	let page_count = match html
		.select("select.page-list option:last-of-type")
		.text()
		.read()
		.rsplit_once(' ')
	{
		Some((_, str)) => str.parse::<i32>().unwrap_or(0),
		None => 0,
	};

	let mut pages: Vec<Page> = Vec::new();

	let first_url = html.select("img.picture").attr("src").read();

	if first_url.contains(BASE_URL) {
		// we know image urls
		for index in 0..page_count {
			let url = format!("{BASE_URL}/viewer{chapter_id}/{}", index + 1);

			pages.push(Page {
				index,
				url,
				..Default::default()
			})
		}
	} else {
		// have to request each page for the image url
		pages.push(Page {
			index: 0,
			url: encode_uri(first_url),
			..Default::default()
		});
		for index in 1..page_count {
			let url = format!("{url}/{}", index + 1);
			let page_html = match Request::new(url.clone(), HttpMethod::Get).html() {
				Ok(html) => html,
				Err(_) => continue,
			};
			let page_url = encode_uri(page_html.select("img.picture").attr("src").read());

			pages.push(Page {
				index,
				url: page_url,
				..Default::default()
			})
		}
	}

	Ok(pages)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	// manga: https://raw.senmanga.com/manga_id
	// chapter: https://raw.senmanga.com/manga_id/chapter/page

	let split = url.split('/').collect::<Vec<&str>>();
	if split.len() >= 3 {
		let manga_id = String::from("/") + split[3];
		let chapter_id = if split.len() > 3 {
			Some(String::from("/") + &split[3..4].join("/"))
		} else {
			None
		};

		let manga = get_manga_details(manga_id).ok();
		let chapter = chapter_id.map(|id| Chapter {
			id,
			..Default::default()
		});

		Ok(DeepLink { manga, chapter })
	} else {
		panic!("unhandled url")
	}
}
