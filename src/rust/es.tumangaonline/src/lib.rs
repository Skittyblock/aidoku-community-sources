#![no_std]
use aidoku::{
	error::Result,
	helpers::substring::Substring,
	prelude::*,
	std::{defaults::defaults_get, html::Node, net::HttpMethod, net::Request, String, Vec},
	Chapter, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

extern crate alloc;
use alloc::{borrow::ToOwned, string::ToString};

mod parser;

static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.114 Safari/537.36";
static BASE_URL: &str = "https://zonatmo.com/";
static BASE_IMAGE_REFERER: &str = "https://zonatmo.com";

#[link(wasm_import_module = "net")]
extern "C" {
	fn set_rate_limit(rate_limit: i32);
	fn set_rate_limit_period(period: i32);
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn initialize() {
	let rate_limit: i32 = match defaults_get("rateLimit") {
		Ok(limit) => limit.as_int().unwrap_or(10) as i32,
		Err(_) => 10,
	};
	set_rate_limit(rate_limit);
	set_rate_limit_period(60);
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut url = BASE_URL.to_owned() + "library?_pg=1&page=" + &page.to_string();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				url.push_str("&filter_by=title&title=");
				url.push_str(
					&filter
						.value
						.as_string()
						.expect("title filter value not string")
						.read(),
				);
			}
			FilterType::Sort => {
				let asc = filter.object.get("ascending").as_bool().unwrap_or(false);
				let idx = filter.object.get("index").as_int().unwrap_or(-1);
				let option = match idx {
					0 => "likes_count",
					1 => "alphabetically",
					2 => "score",
					3 => "creation",
					4 => "release_date",
					5 => "num_chapters",
					_ => continue,
				};
				url.push_str("&order_item=");
				url.push_str(option);
				url.push_str("&order_type=");
				url.push_str(if asc { "asc" } else { "desc" });
			}
			FilterType::Genre => {
				let option = match filter.value.as_int().unwrap_or(-1) {
					0 => "&exclude_genders[]=",
					1 => "&genders[]=",
					_ => continue,
				};
				let id = filter
					.object
					.get("id")
					.as_string()
					.expect("filter genre doesn't have id")
					.read();
				url.push_str(option);
				url.push_str(&id);
			}
			FilterType::Check => {
				let value = match filter.value.as_int() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let option = match filter.name.as_str() {
					"Webcomic" => "&webcomic=",
					"Yonkoma" => "&yonkoma=",
					"Amateur" => "&amateur=",
					"Erótico" => "&erotic=",
					_ => continue,
				};
				url.push_str(option);
				url.push_str(if value == 0 { "false" } else { "true" });
			}
			FilterType::Select => {
				let value = match filter.value.as_int() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let (option, choice) = match filter.name.as_str() {
					"Filtrar por" => (
						"&filter_by=",
						match value {
							0 => "title",
							1 => "author",
							2 => "company",
							_ => continue,
						},
					),
					"Demografía" => (
						"&demography=",
						match value {
							1 => "seinen",
							2 => "shoujo",
							3 => "shounen",
							4 => "josei",
							5 => "kodomo",
							_ => continue,
						},
					),
					"Estado de traducción" => (
						"&translation_status=",
						match value {
							1 => "publishing",
							2 => "ended",
							3 => "cancelled",
							4 => "on_hold",
							_ => continue,
						},
					),
					"Estado de serie" => (
						"&status=",
						match value {
							1 => "publishing",
							2 => "ended",
							3 => "cancelled",
							4 => "on_hold",
							_ => continue,
						},
					),
					"Tipo" => (
						"&type=",
						match value {
							1 => "manga",
							2 => "manhua",
							3 => "manhwa",
							4 => "novel",
							5 => "one_shot",
							6 => "doujinshi",
							7 => "oel",
							_ => continue,
						},
					),
					_ => continue,
				};
				url.push_str(option);
				url.push_str(choice);
			}
			_ => continue,
		}
	}

	parser::parse_manga_list(url)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	match listing.name.as_str() {
		"Latest" => {
			let url = String::from("https://zonatmo.com/library?order_item=creation&order_dir=desc&filter_by=title&_pg=1&page=") + &page.to_string();
			parser::parse_manga_list(url)
		}
		"Popular" => {
			let url = String::from("https://zonatmo.com/library?order_item=likes_count&order_dir=desc&filter_by=title&_pg=1&page=") + &page.to_string();
			parser::parse_manga_list(url)
		}
		_ => get_manga_list(Vec::new(), page),
	}
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = if id.starts_with("http") {
		id.clone()
	} else {
		format!("{BASE_URL}{id}")
	};
	let html = Request::new(&url, HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.header("Referer", BASE_URL)
		.html()?;

	let cover = html.select(".book-thumbnail").attr("src").read();
	let title = html.select("h1.element-title").first().own_text().read();
	let title_elements = html.select("h5.card-title");
	let author = title_elements
		.first()
		.attr("title")
		.read()
		.replace(", ", "");
	let artist = title_elements.last().attr("title").read().replace(", ", "");
	let description = html.select("p.element-description").text().read();

	let categories = html
		.select("a.py-2")
		.array()
		.map(|x| {
			x.as_node()
				.expect("node array element should be a node")
				.text()
				.read()
		})
		.collect::<Vec<String>>();

	let status_text = html.select("span.book-status").text().read();
	let status = match status_text.as_str() {
		"Publicándose" => MangaStatus::Ongoing,
		"Finalizado" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};

	let nsfw = if !html.select("i.fa-heartbeat").array().is_empty() {
		MangaContentRating::Nsfw
	} else if categories.iter().any(|x| x == "Ecchi") {
		MangaContentRating::Suggestive
	} else {
		MangaContentRating::Safe
	};

	let type_text = html.select("h1.book-type").text().read();
	let viewer = match type_text.as_str() {
		"MANHWA" => MangaViewer::Scroll,
		"MANHUA" => MangaViewer::Scroll,
		_ => MangaViewer::Rtl,
	};

	Ok(Manga {
		id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories,
		status,
		nsfw,
		viewer,
	})
}

fn parse_chapter(element: Node) -> Chapter {
	let url = element
		.select("div.row > .text-right > a")
		.attr("href")
		.read();
	let id = url.strip_prefix(BASE_URL).unwrap_or(&url).to_owned();

	let scanlator = element.select("div.col-md-6.text-truncate").text().read();

	let date_updated = element
		.select("span.badge.badge-primary.p-2")
		.first()
		.text()
		.as_date("yyyy-MM-dd", None, None);

	Chapter {
		id,
		url,
		scanlator,
		date_updated,
		..Default::default()
	}
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = if id.starts_with("http") {
		id
	} else {
		format!("{BASE_URL}{id}")
	};
	let html = Request::new(url, HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.header("Referer", BASE_URL)
		.html()?;

	let chapter_elements = html.select("div.chapters > ul.list-group li.p-0.list-group-item");

	let mut chapters: Vec<Chapter> = Vec::new();

	if chapter_elements.array().is_empty() {
		// one shot
		let elements = html
			.select("div.chapter-list-element > ul.list-group li.list-group-item")
			.array();
		for element in elements {
			let mut chapter = parse_chapter(
				element
					.as_node()
					.expect("html array element should be a node"),
			);
			chapter.title = String::from("One Shot");
			chapters.push(chapter);
		}
	} else {
		for element in chapter_elements.array() {
			let element = element
				.as_node()
				.expect("html array element should be a node");
			let title = element.select("div.col-10.text-truncate").text().read();
			let chapter_num = {
				let num_text = element.select("a.btn-collapse").text().read();
				let half = num_text.substring_after("Capítulo ").unwrap_or(&num_text);
				half.chars()
					.filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.')
					.collect::<String>()
					.split(' ')
					.collect::<Vec<&str>>()
					.into_iter()
					.map(|a| a.parse::<f32>().unwrap_or(0.0))
					.find(|a| *a > 0.0)
					.unwrap_or(0.0)
			};

			let scanlations = element.select("ul.chapter-list > li");
			for scanlation in scanlations.array() {
				let mut chapter = parse_chapter(
					scanlation
						.as_node()
						.expect("html array element should be a node"),
				);
				chapter.title = title.clone();
				chapter.chapter = chapter_num;
				chapters.push(chapter);
			}
		}
	}

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = if chapter_id.starts_with("http") {
		chapter_id
	} else {
		format!("{BASE_URL}{chapter_id}")
	};
	let mut html = Request::new(url, HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.header("Referer", BASE_URL)
		.html()?;

	let uri = html.base_uri().read();
	if uri.contains("/paginated") {
		// switch to cascade for full image list
		html = Request::new(uri.replace("/paginated", "/cascade"), HttpMethod::Get)
			.header("User-Agent", USER_AGENT)
			.header("Referer", BASE_URL)
			.html()?;
	}

	let mut pages: Vec<Page> = Vec::new();
	let page_elements = html.select("div.viewer-container img");

	for (index, element) in page_elements.array().enumerate() {
		let element = element
			.as_node()
			.expect("html array element should be a node");
		let url = if element.has_attr("data-src") {
			element.attr("abs:data-src").read()
		} else {
			element.attr("abs:src").read()
		};
		pages.push(Page {
			index: index as i32,
			url,
			..Default::default()
		})
	}

	Ok(pages)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
		.header("User-Agent", USER_AGENT)
		.header("Referer", BASE_IMAGE_REFERER);
}
