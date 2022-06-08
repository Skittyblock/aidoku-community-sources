#![no_std]
mod helper;
mod parser;
extern crate alloc;

use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	prelude::*,
	std::{
		html::Node,
		json,
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult,
	MangaStatus, MangaViewer, Page,
};
use helper::{
	capitalize_first_letter, category_parser, extract_f32_from_string, get_lang_code,
	text_with_newlines, urlencode,
};
use parser::{convert_time, parse_manga_list};

static mut CACHED_MANGA_ID: Option<String> = None;
static mut CACHED_MANGA: Option<Vec<u8>> = None;

fn cache_manga_page(id: &str) {
	if unsafe { CACHED_MANGA_ID.is_some() } && unsafe { CACHED_MANGA_ID.clone().unwrap() } == id {
		return;
	}

	unsafe {
		CACHED_MANGA = Some(
			Request::new(
				format!("https://otakusan.net{id}").as_str(),
				HttpMethod::Get,
			)
			.data(),
		);
		CACHED_MANGA_ID = Some(String::from(id));
	};
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut title = String::new();
	let mut tags: Vec<String> = Vec::with_capacity(49); // Number of filters available
	let mut search_request = false;
	for filter in filters {
		match filter.kind {
			FilterType::Title => title = urlencode(filter.value.as_string()?.read()),
			FilterType::Genre | FilterType::Check => match filter.value.as_int().unwrap_or(-1) {
				1 => tags.push(filter.object.get("id").as_string()?.read()),
				_ => continue,
			},

			_ => continue,
		}
	}
	tags.resize(tags.len(), String::new());
	let resp = if !title.is_empty() {
		let url = format!("https://otakusan.net/Home/Search?search={title}");
		search_request = true;
		Request::new(&url, HttpMethod::Get).html()
	} else {
		let mut request = format!(
			"Lang={}&Page={page}&Type=Include&Dir=NewPostedDate",
			get_lang_code()
		);
		for (idx, tag) in tags.iter().enumerate() {
			request.push_str(format!("&FilterCategory[{idx}]={tag}").as_str());
		}
		Request::new("https://otakusan.net/Manga/Newest", HttpMethod::Post)
			.body(request.as_bytes())
			.header(
				"Content-Type",
				"application/x-www-form-urlencoded; charset=UTF-8",
			)
			.header("X-Requested-With", "XMLHttpRequest")
			.header("Referer", "https://otakusan.net/")
			.header("Origin", "https://otakusan.net")
			.html()
	};
	let (manga, has_more) = parse_manga_list!(resp.select("div.mdl-card").array());
	Ok(MangaPageResult {
		manga,
		has_more: if search_request { false } else { has_more },
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let url = match listing.name.as_str() {
		"Completed" => "https://otakusan.net/Manga/CompletedNewest",
		"New Titles" => "https://otakusan.net/Manga/NewTitleNewest",
		"For Boys" => "https://otakusan.net/Manga/ForBoyNewest",
		"For Girls" => "https://otakusan.net/Manga/ForGirlNewest",
		_ => {
			return Err(AidokuError {
				reason: AidokuErrorKind::Unimplemented,
			})
		}
	};
	let request = format!(
		"Lang={}&Page={page}&Type=Include&Dir=NewPostedDate",
		get_lang_code()
	);
	let resp = &Request::new(url, HttpMethod::Post)
		.body(request.as_bytes())
		.header(
			"Content-Type",
			"application/x-www-form-urlencoded; charset=UTF-8",
		)
		.header("X-Requested-With", "XMLHttpRequest")
		.header("Referer", "https://otakusan.net/")
		.header("Origin", "https://otakusan.net")
		.html();
	let (manga, has_more) = parse_manga_list!(resp.select("div.mdl-card").array());
	Ok(MangaPageResult { manga, has_more })
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	cache_manga_page(&id);
	let html = unsafe { Node::new(&CACHED_MANGA.clone().unwrap()) };
	let url = format!("https://otakusan.net{id}");
	let title = capitalize_first_letter(String::from(
		html.select("h1.title.text-lg-left.text-overflow-2-line")
			.text()
			.read()
			.trim(),
	));
	let cover = html
		.select("div.container-3-4.background-contain img")
		.attr("src")
		.read()
		.replace("http:", "https:");
	let description = text_with_newlines(html.select("div.summary p"));
	let author = capitalize_first_letter(String::from(
		html.select("tr:contains(Tác Giả) a.capitalize[href*=Author]")
			.attr("title")
			.read()
			.trim(),
	));
	let categories = html
		.select("div.genres a")
		.array()
		.map(|val| val.as_node().text().read())
		.collect::<Vec<String>>();
	let status = match html
		.select("tr:contains(Tình Trạng) td")
		.array()
		.get(0)
		.as_node()
		.text()
		.read()
		.trim()
	{
		"Ongoing" => MangaStatus::Ongoing,
		"Done" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};
	let viewer = match html
		.select("tr:contains(Loại Truyện) td")
		.array()
		.get(0)
		.as_node()
		.text()
		.read()
		.trim()
	{
		"Manhua" | "Manhwa" => MangaViewer::Scroll,
		"VnComic" => MangaViewer::Ltr,
		_ => MangaViewer::Rtl,
	};
	let (mut nsfw, _) = category_parser(&categories);
	if html.select("div.alert:contains(18 tuổi)").array().len() > 0 {
		nsfw = MangaContentRating::Nsfw;
	}
	Ok(Manga {
		id,
		cover,
		title: String::from(title.trim()),
		author,
		artist: String::new(),
		description: String::from(description.trim()),
		url,
		categories,
		status,
		nsfw,
		viewer,
	})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	cache_manga_page(&id);
	let html = unsafe { Node::new(&CACHED_MANGA.clone().unwrap()) };
	let scanlator = html
		.select("tr:contains(Nhóm Dịch) a")
		.attr("title")
		.read()
		.replace(" collections", "");
	let mut lang = html
		.select("h1.title img.flag")
		.attr("src")
		.read()
		.replace("https://ipdata.co/flags/", "")
		.replace(".png", "");
	lang = String::from(match lang.as_str() {
		"us" => "en",
		"vn" => "vi",
		_ => lang.as_str(),
	});

	let node = html.select("tr.chapter");
	let elems = node.array();
	let mut chapters: Vec<Chapter> = Vec::with_capacity(elems.len());
	for elem in elems {
		let elem_node = elem.as_node();
		let cells_node = elem_node.select("td");
		let cells = cells_node.array();
		let mut chapter: Chapter = Chapter {
			id: String::new(),
			title: String::new(),
			volume: -1.0,
			chapter: -1.0,
			date_updated: -1.0,
			scanlator: if scanlator.as_str() == "Unknown" {
				String::new()
			} else {
				scanlator.clone()
			},
			url: String::new(),
			lang: lang.clone(),
		};
		for (idx, cell) in cells.enumerate() {
			let node = cell.as_node();
			match idx {
				0 => {
					chapter.chapter = extract_f32_from_string(String::new(), node.text().read());
				}
				1 => {
					let anchor = node.select("a");
					chapter.id = anchor.attr("href").read();
					chapter.title = String::from(anchor.text().read().trim());
					chapter.url = format!("https://otakusan.net{}", chapter.id);
				}
				3 => {
					chapter.date_updated = convert_time(String::from(node.text().read().trim()));
				}
				_ => continue,
			}
		}
		chapters.push(chapter);
	}
	Ok(chapters)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	let resp = Request::new(
		format!("https://otakusan.net{id}").as_str(),
		HttpMethod::Get,
	)
	.html();
	let vi = resp.select("#dataip").attr("value").read();
	let numeric_id = resp.select("#inpit-c").attr("data-chapter-id").read();
	let json = Request::new(
		"https://otakusan.net/Manga/CheckingAlternate",
		HttpMethod::Post,
	)
	.body(format!("chapId={numeric_id}").as_bytes())
	.header(
		"Content-Type",
		"application/x-www-form-urlencoded; charset=UTF-8",
	)
	.header("X-Requested-With", "XMLHttpRequest")
	.header("Referer", format!("https://otakusan.net{id}").as_str())
	.header("Origin", "https://otakusan.net")
	.json();
	let json_object = json.as_object()?;
	let raw_pages_arr_value = json_object.get("Content");
	let raw_pages_arr = if raw_pages_arr_value.is_none() {
		let json = Request::new("https://otakusan.net/Manga/UpdateView", HttpMethod::Post)
			.body(format!("chapId={numeric_id}").as_bytes())
			.header(
				"Content-Type",
				"application/x-www-form-urlencoded; charset=UTF-8",
			)
			.header("X-Requested-With", "XMLHttpRequest")
			.header("Referer", format!("https://otakusan.net{id}").as_str())
			.header("Origin", "https://otakusan.net")
			.json();
		let json_object = json.as_object()?;
		let raw_pages_arr_value = json_object.get("view");
		raw_pages_arr_value.as_string()?.read()
	} else {
		raw_pages_arr_value.as_string()?.read()
	};
	let pages = json::parse(raw_pages_arr.as_bytes()).as_array()?;
	let mut page_arr: Vec<Page> = Vec::with_capacity(pages.len());
	for (index, page) in pages.enumerate() {
		let url = parser::url_replacer(page.as_string()?.read(), vi.clone());
		page_arr.push(Page {
			index: index as i32,
			url,
			base64: String::new(),
			text: String::new(),
		});
	}
	Ok(page_arr)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
		.header("Referer", "https://otakusan.net/")
		.header("Origin", "https://otakusan.net");
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	if url.contains("manga-detail") {
		let id = String::from(&url[20..]);
		Ok(DeepLink {
			manga: Some(get_manga_details(id)?),
			chapter: None,
		})
	} else if url.contains("chapter") {
		let resp = Request::new(&url, HttpMethod::Get).html();
		let breadcrumbs_node = resp.select("a.itemcrumb.active");
		let manga_id = breadcrumbs_node.attr("href").read();
		let manga = Some(get_manga_details(manga_id)?);
		let chapter = Some(Chapter {
			id: String::from(&url[20..]),
			title: String::new(),
			volume: -1.0,
			chapter: -1.0,
			date_updated: -1.0,
			scanlator: String::new(),
			url,
			lang: String::new(),
		});
		Ok(DeepLink { manga, chapter })
	} else {
		Err(AidokuError {
			reason: AidokuErrorKind::Unimplemented,
		})
	}
}
