use alloc::string::ToString;
use aidoku::{
	error::Result,
	helpers::substring::Substring,
	prelude::*,
	std::{
		net::{HttpMethod, Request},
		html::Node,
		String, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaViewer,
	Page,
};
use crate::helper::{get_chapter_number, get_search_url, manga_status};

static COOKIE_NAME: &str = "MWCookie=";
static mut STORED_COOKIE: Option<String> = None;

fn get_html_with_cookie(url: String, referer: Option<&str>) -> Result<Node> {
	let cookie = unsafe {
		match STORED_COOKIE {
			Some(ref c) => c.clone(),
			None => COOKIE_NAME.to_string()
		}
	};
	let request = Request::new(&url, HttpMethod::Get)
		.header("Cookie", &cookie);

	let request = if let Some(ref_url) = referer {
		request.header("referer", ref_url)
	} else {
		request
	};

	let html = request.html()?;

	if let (Some(start), Some(end)) = (
		html.outer_html().to_string().find(COOKIE_NAME).map(|i| i + COOKIE_NAME.len()),
		html.outer_html().to_string()[html.outer_html().to_string().find(COOKIE_NAME).map_or(0, |i| i + COOKIE_NAME.len())..].find(";")
	) {
		let new_cookie = format!("{}{}",COOKIE_NAME, &html.outer_html().to_string()[start..start + end]);
		unsafe { STORED_COOKIE = Some(new_cookie.clone()) };

		let final_request = Request::new(&url, HttpMethod::Get)
			.header("Cookie", &new_cookie);

		return if let Some(ref_url) = referer {
			final_request.header("referer", ref_url).html()
		} else {
			final_request.html()
		};
	}

	Ok(html)
}

pub fn parse_manga_list(
	base_url: String,
	filters: Vec<Filter>,
	page: i32,
) -> Result<MangaPageResult> {
	let mut included_tags: Vec<String> = Vec::new();
	let mut title: String = String::new();
	let status_options = ["", "ongoing", "completed", "dropped", "paused", "canceled"];
	let type_options = [
		"",
		"manga",
		"manhua",
		"manhwa",
		"oneshot",
		"thai",
		"vietnamese",
	];
	let mut manga_type: String = String::new();
	let mut status: String = String::new();
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				title = filter.value.as_string()?.read();
			}
			FilterType::Genre => match filter.value.as_int().unwrap_or(-1) {
				1 => included_tags.push(filter.object.get("id").as_string()?.read()),
				_ => continue,
			},

			FilterType::Select => {
				let index = filter.value.as_int().unwrap_or(-1) as usize;
				match filter.name.as_str() {
					"Stato" => status = String::from(status_options[index]),
					"Tipo" => manga_type = String::from(type_options[index]),
					_ => continue,
				}
			}
			_ => continue,
		};
	}
	let url = get_search_url(base_url, title, page, included_tags, status, manga_type);
	parse_manga_listing(url, String::new(), page)
}

pub fn parse_manga_listing(
	base_url: String,
	listing_name: String,
	page: i32,
) -> Result<MangaPageResult> {
	let list_url = if !base_url.contains("archive") {
		match listing_name.as_str() {
			"Più letti" => format!("{base_url}/archive?sort=most_read&page={page}"),
			"Più recenti" => format!("{base_url}/archive?sort=newest&page={page}"),
			_ => format!("{base_url}/archive?page={page}"),
		}
	} else {
		base_url
	};

	let html = get_html_with_cookie(list_url, None)?;

	let mut count = 0;
	let mut mangas: Vec<Manga> = Vec::new();
	for manga in html.select(".comics-grid .entry").array() {
		let manga_node = manga.as_node().expect("Failed to get manga as node");
		let title = manga_node.select(".manga-title").text().read();
		let url = manga_node.select(".manga-title").attr("href").read();
		let id = String::from(url.substring_after("manga/").unwrap_or_default());
		let cover = manga_node.select("img").attr("src").read();
		mangas.push(Manga {
			id,
			cover,
			title,
			url,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Rtl,
			..Default::default()
		});
		count += 1;
	}
	let has_more = count == 16;

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_manga_details(base_url: String, id: String) -> Result<Manga> {
	let url = format!("{base_url}/manga/{id}");
	let html = get_html_with_cookie(url.clone(), None)?;
	let title = html.select("h1").text().read();
	let cover = html.select(".single-comic .thumb img").attr("src").read();
	let author = html
		.select("div.info > div.meta-data.row.px-1 > div:nth-child(3) a")
		.array()
		.map(|val| val.as_node().expect("Failed to get author").text().read())
		.collect::<Vec<String>>()
		.join(", ");
	let artist = html
		.select("div.info > div.meta-data.row.px-1 > div:nth-child(4) a")
		.array()
		.map(|val| val.as_node().expect("Failed to get author").text().read())
		.collect::<Vec<String>>()
		.join(", ");
	let description = html.select("#noidungm").text().read();
	let categories = html
		.select("div.info > div.meta-data.row.px-1 > div:nth-child(2) a")
		.array()
		.map(|val| val.as_node().expect("Failed to get author").text().read())
		.collect::<Vec<String>>();
	let status = manga_status(
		html.select("div.info > div.meta-data.row.px-1 > div:nth-child(6) > a")
			.text()
			.read(),
	);
	let nsfw = if base_url == "https://www.mangaworldadult.com"
		|| categories
			.iter()
			.any(|v| *v == "Ecchi" || *v == "Hentai" || *v == "Maturo")
	{
		MangaContentRating::Nsfw
	} else {
		MangaContentRating::Safe
	};
	let manga_type = html
		.select("div.info > div.meta-data.row.px-1 > div:nth-child(5) > a")
		.text()
		.read();
	let viewer = match manga_type.as_str() {
		"Manhua" => MangaViewer::Ltr,
		"Manhwa" => MangaViewer::Scroll,
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

pub fn parse_chapter_list(base_url: String, id: String) -> Result<Vec<Chapter>> {
	let url = format!("{base_url}/manga/{}", id);
	let html = get_html_with_cookie(url.clone(), Some(&url))?;

	let mut chapters: Vec<Chapter> = Vec::new();
	for chapter in html.select(".chapters-wrapper .chap").array() {
		let chapter_node = chapter.as_node().expect("Failed to get chapter as node");
		let title = chapter_node.select("span").text().read();
		let chapter_number = get_chapter_number(title.clone());
		let chapter_url = chapter_node.attr("href").read();
		let chapter_id = String::from(
			chapter_url
				.clone()
				.substring_after("read/")
				.unwrap_or_default(),
		);
		let date_updated = chapter_node
			.select("i")
			.text()
			.0
			.as_date("d MMMM yyyy", Some("it"), None)
			.unwrap_or(-1.0);
		chapters.push(Chapter {
			id: chapter_id,
			title,
			volume: -1.0,
			chapter: chapter_number,
			date_updated,
			scanlator: String::new(),
			url: chapter_url,
			lang: String::from("it"),
		});
	}
	Ok(chapters)
}

pub fn parse_page_list(
	base_url: String,
	manga_id: String,
	chapter_id: String,
) -> Result<Vec<Page>> {
	let url = format!("{base_url}/manga/{manga_id}/read/{chapter_id}/?style=list");
	let html = get_html_with_cookie(url.clone(), Some(&url))?;

	let mut pages: Vec<Page> = Vec::new();

	for (at, page) in html.select("#page img").array().enumerate() {
		let page_node = page.as_node().expect("Failed to get page as node");
		let page_url = page_node.attr("src").read();
		pages.push(Page {
			index: at as i32,
			url: page_url,
			base64: String::new(),
			text: String::new(),
		});
	}
	Ok(pages)
}

pub fn modify_image_request(base_url: String, request: Request) {
	request.header("Referer", &base_url);
}

pub fn handle_url(base_url: String, url: String) -> Result<DeepLink> {
	let id = String::from(url.substring_after("manga/").unwrap_or_default());
	Ok(DeepLink {
		manga: parse_manga_details(base_url, id).ok(),
		chapter: None,
	})
}
