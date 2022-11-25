use aidoku::{
	error::Result, prelude::format, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

use crate::helper::*;

pub fn parse_manga_list(
	base_url: String,
	filters: Vec<Filter>,
	page: i32,
) -> Result<MangaPageResult> {
	let mut title: String = String::new();
	let tag_list = genres();
	let mut tag: String = String::new();
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				title = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				if filter.name.as_str() == "Genres" {
					let index = filter.value.as_int()? as usize;
					match index {
						0 => continue,
						_ => tag = String::from(tag_list[index]),
					}
				}
			}
			_ => continue,
		}
	}

	if !title.is_empty() {
		let mut mangas: Vec<Manga> = Vec::new();
		let url = format!("{}/service/search", base_url);
		let request = Request::new(url.as_str(), HttpMethod::Post);
		let body_data = format!("dataType=json&phrase={}", title);
		let json = request
			.header("X-Requested-With", "XMLHttpRequest")
			.header("Content-Type", "application/x-www-form-urlencoded")
			.body(body_data.as_bytes())
			.json()
			.as_object()?;

		let data = json.get("manga").as_array()?;
		for manga in data {
			let manga_node = manga.as_object()?;
			let title = manga_node.get("title").as_string()?.read();
			let id = manga_node.get("url").as_string()?.read();
			let url = get_full_url(manga_node.get("url").as_string()?.read());
			let cover = get_full_url(manga_node.get("image").as_string()?.read());
			mangas.push(Manga {
				id,
				cover,
				title,
				author: String::new(),
				artist: String::new(),
				description: String::new(),
				url,
				categories: Vec::new(),
				status: MangaStatus::Unknown,
				nsfw: MangaContentRating::Safe,
				viewer: MangaViewer::Default,
			});
		}

		let total = json.get("total").as_int().unwrap_or(0) as i32;
		Ok(MangaPageResult {
			manga: mangas,
			has_more: page < total,
		})
	} else if title.is_empty() && tag.is_empty() {
		parse_manga_listing(
			base_url.clone(),
			format!("{}/latest-releases/{}", base_url, page),
			String::from("Latest"),
		)
	} else {
		let mut mangas: Vec<Manga> = Vec::new();
		let url = format!("{}/category/{}/watch/{}", base_url, tag, page);
		let html = Request::new(url.as_str(), HttpMethod::Get).html();
		for manga in html.select(".filter-results .mb-lg").array() {
			let manga_node = manga.as_node();
			let title = manga_node.select("h2").text().read();
			let id = get_manga_id(manga_node.select("a").attr("href").read());
			let cover = base_url.clone() + &get_image_src(manga_node);
			mangas.push(Manga {
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
				viewer: MangaViewer::Default,
			});
		}
		let last_page = html.select("div.ui.pagination.menu li").text().read();
		let has_more = last_page.contains('»');
		Ok(MangaPageResult {
			manga: mangas,
			has_more,
		})
	}
}

pub fn parse_manga_listing(
	base_url: String,
	url: String,
	list_type: String,
) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	let html = Request::new(url.as_str(), HttpMethod::Get).html();
	if list_type == "Hot" {
		for manga in html.select("#manga-hot-updates .item").array() {
			let manga_node = manga.as_node();
			let title = manga_node.select("strong").text().read();
			let id = get_manga_id(manga_node.select("a").attr("href").read());
			let cover =
				base_url.clone() + manga_node.select("img").attr("src").read().as_str().trim();
			mangas.push(Manga {
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
				viewer: MangaViewer::Default,
			});
		}
		Ok(MangaPageResult {
			manga: mangas,
			has_more: false,
		})
	} else if list_type == "Popular" {
		for manga in html.select(".filter-results .mb-lg").array() {
			let manga_node = manga.as_node();
			let title = manga_node.select("h2").text().read();
			let id = get_manga_id(manga_node.select("a").attr("href").read());
			let cover =
				base_url.clone() + manga_node.select("img").attr("src").read().as_str().trim();
			mangas.push(Manga {
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
				viewer: MangaViewer::Default,
			});
		}
		let last_page = html.select("div.ui.pagination.menu li").text().read();
		let has_more = last_page.contains('»');
		Ok(MangaPageResult {
			manga: mangas,
			has_more,
		})
	} else if list_type == "Latest" {
		for manga in html.select(".latest-updates .poster.poster-xs").array() {
			let manga_node = manga.as_node();
			let title = manga_node.select("h2").text().read();
			let id = get_manga_id(manga_node.select("a").attr("href").read());
			let cover =
				base_url.clone() + manga_node.select("img").attr("data-src").read().as_str();
			mangas.push(Manga {
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
				viewer: MangaViewer::Default,
			});
		}
		let last_page = html.select("div.ui.pagination.menu li").text().read();
		let has_more = last_page.contains('»');
		Ok(MangaPageResult {
			manga: mangas,
			has_more,
		})
	} else {
		for manga in html.select(".clearfix.mb-0 li").array() {
			let manga_node = manga.as_node();
			let title = manga_node.select("h2").text().read();
			let id = get_manga_id(manga_node.select("a").attr("href").read());
			let cover = base_url.clone()
				+ manga_node
					.select("img")
					.attr("data-src")
					.read()
					.as_str()
					.trim();
			mangas.push(Manga {
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
				viewer: MangaViewer::Default,
			});
		}
		Ok(MangaPageResult {
			manga: mangas,
			has_more: false,
		})
	}
}

pub fn parse_manga_details(base_url: String, raw_id: String) -> Result<Manga> {
	let id = get_full_url(raw_id);
	let html = Request::new(id.as_str(), HttpMethod::Get).html();
	let title = html.select(".page-title").text().read();
	let cover = base_url + html.select(".image img").attr("src").read().as_str().trim();
	let author = html.select("#first_episode small").text().read();
	let artist = html.select("#last_episode small").text().read();
	let description = html
		.select(".series-summary-wrapper span")
		.first()
		.text()
		.read();
	let status = manga_status(
		html.select("div.series-genres")
			.text()
			.read()
			.to_uppercase(),
	);
	let mut categories = Vec::new();
	let mut nsfw = MangaContentRating::Safe;
	for node in html.select(".series-summary-wrapper a").array() {
		let category = node.as_node().text().read();
		if category.clone().as_str() == "Mature" || category.clone().as_str() == "Ecchi" {
			nsfw = MangaContentRating::Nsfw;
		}
		categories.push(category.clone());
	}
	let manga_type = html
		.select("table td:eq(0) div")
		.text()
		.read()
		.replace("Type", "")
		.replace(' ', "");
	let viewer = match manga_type.as_str() {
		"Japanese" => MangaViewer::Rtl,
		"Korean" => MangaViewer::Scroll,
		_ => MangaViewer::Rtl,
	};
	Ok(Manga {
		id: id.clone(),
		cover,
		title,
		author,
		artist,
		description,
		url: id,
		categories,
		status,
		nsfw,
		viewer,
	})
}

pub fn parse_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	let html = Request::new(id.as_str(), HttpMethod::Get).html();
	for chapter in html.select(".season_start").array() {
		let chapter_node = chapter.as_node();
		let title = String::from(chapter_node.select("h6").text().read().as_str().trim());
		let chapter_id = get_full_url(chapter_node.select("a").attr("href").read());
		let chapter_url = get_full_url(chapter_node.select("a").attr("href").read());
		let chapter_number = get_chapter_number(chapter_id.clone());
		let date = get_date(chapter_node.select(".episode-date").text().read());
		chapters.push(Chapter {
			id: chapter_id,
			title,
			volume: -1.0,
			chapter: chapter_number,
			date_updated: date,
			scanlator: String::new(),
			url: chapter_url,
			lang: String::from("en"),
		});
	}
	Ok(chapters)
}

pub fn parse_page_list(id: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	let html = Request::new(id.as_str(), HttpMethod::Get).html();
	for (index, page) in html.select("div.ch-images img").array().enumerate() {
		let page_node = page.as_node();
		let page_url = get_full_url(page_node.attr("src").read());
		pages.push(Page {
			index: index.try_into().unwrap_or(-1),
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
	Ok(DeepLink {
		manga: Some(parse_manga_details(base_url, url)?),
		chapter: None,
	})
}
