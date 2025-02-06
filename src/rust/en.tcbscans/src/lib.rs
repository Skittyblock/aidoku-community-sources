#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::HttpMethod, net::Request, String, Vec},
	Chapter, DeepLink, Filter, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

const BASE_URL: &str = "https://tcbonepiecechapters.com";

#[get_manga_list]
fn get_manga_list(_filters: Vec<Filter>, _page: i32) -> Result<MangaPageResult> {
	let html = Request::new(format!("{}/projects", BASE_URL), HttpMethod::Get).html()?;

	let elements = html.select(".bg-card.border.border-border.rounded.p-3.mb-3");

	let mut manga: Vec<Manga> = Vec::new();

	for element in elements.array() {
		let item = element.as_node().expect("html array not an array of nodes");

		let title_element = item.select("a.mb-3.text-white.text-lg.font-bold");
		let id = title_element.attr("href").read();
		let title = title_element.text().read();
		let cover = item
			.select(".w-24.h-24.object-cover.rounded-lg")
			.attr("src")
			.read();

		manga.push(Manga {
			id,
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: String::new(),
			categories: Vec::new(),
			status: MangaStatus::Unknown,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Rtl,
		})
	}

	Ok(MangaPageResult {
		manga,
		has_more: false,
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{}{}", BASE_URL, id);
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;

	let element = html.select(".order-1.bg-card.border.border-border.rounded.py-3");
	let cover = element
		.select(".flex.items-center.justify-center img")
		.attr("src")
		.read();
	let title = element.select(".my-3.font-bold.text-3xl").text().read();
	let description = element.select(".leading-6.my-3").text().read();

	Ok(Manga {
		id,
		cover,
		title,
		author: String::from("TCB Scans"),
		artist: String::new(),
		description,
		url,
		categories: Vec::new(),
		status: MangaStatus::Unknown,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Rtl,
	})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}{}", BASE_URL, id);
	let html = Request::new(url, HttpMethod::Get).html()?;

	let elements = html.select(".bg-card.border.border-border.rounded.p-3.mb-3");

	let mut chapters: Vec<Chapter> = Vec::new();

	for element in elements.array() {
		let item = element.as_node().expect("html array not an array of nodes");

		let title = item.select(".text-lg.font-bold:not(.flex)").text().read();
		let subtitle = item.select(".text-gray-500").text().read();
		let url_path = item.attr("href").read();
		let url = format!("{}{}", BASE_URL, url_path);

		let chapter = match title.rsplit_once(' ') {
			Some((_, str)) => str.parse::<f32>().unwrap_or(-1.0),
			None => -1.0,
		};

		chapters.push(Chapter {
			id: url_path,
			title: subtitle,
			volume: -1.0,
			chapter,
			date_updated: -1.0,
			scanlator: String::from("TCB Scans"),
			url,
			lang: String::from("en"),
		})
	}

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{}{}", BASE_URL, chapter_id);
	let html = Request::new(url, HttpMethod::Get).html()?;

	let elements = html.select(".flex.flex-col.items-center.justify-center picture img");

	let mut pages: Vec<Page> = Vec::new();

	for (index, element) in elements.array().enumerate() {
		let item = element.as_node().expect("html array not an array of nodes");
		let url = item.attr("src").read();

		pages.push(Page {
			index: index as i32,
			url,
			base64: String::new(),
			text: String::new(),
		})
	}

	Ok(pages)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	// BASE_URL/mangas/x/x
	// BASE_URL/chapters/x/x -> not handled because no manga id
	// todo: can get manga url from
	// ".flex.items-center.justify-center.my-6.gap-2.text-sm.font-bold a".last()
	let split = url.split('/').collect::<Vec<&str>>();
	if split.len() > 3 && split[3] == "mangas" {
		let id = String::from("/") + &split[3..].join("/");
		return Ok(DeepLink {
			manga: get_manga_details(id).ok(),
			chapter: None,
		});
	}
	panic!("unhandled url");
}
