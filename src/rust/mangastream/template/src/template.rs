use aidoku::{
	error::Result, std::String, std::Vec, std::json::parse, std::net::Request, std::net::HttpMethod,
	Filter, FilterType, Listing, Manga, MangaPageResult, Page, MangaStatus, MangaContentRating, MangaViewer, Chapter, DeepLink,
};

use crate::helper::*;

// parse the homepage and filters
pub fn parse_manga_list(base_url: String, traverse_pathname: String, filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut included_tags: Vec<String> = Vec::new();
	let mut status: String = String::new();
	let mut title: String = String::new();
	let mut manga_type: String = String::new();
	let status_options = ["", "ongoing", "completed", "hiatus"];
	let type_options = ["", "manga", "manhwa", "manhua", "comic"];
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				title = filter.value.as_string()?.read();
			},
			FilterType::Genre => {
				match filter.value.as_int().unwrap_or(-1) {
					1 => {
						if let Ok(id) = filter.object.get("id").as_string() {
							included_tags.push(id.read());
						}
					},
					_ => continue,
				}
			},
			FilterType::Select => {
				let index = filter.value.as_int().unwrap_or(-1) as usize;
				match filter.name.as_str() {
					"Status" => status = String::from(status_options[index]),
					"Type" => manga_type = String::from(type_options[index]), 
					_ => continue,
				}
			}
			_ => continue,
		};
	}
	let mut mangas: Vec<Manga> = Vec::new();
	let url = get_search_url(base_url, title, page, included_tags, status, manga_type, traverse_pathname);
	let html = Request::new(url.as_str(), HttpMethod::Get).html();
	for item in html.select(".listupd .bsx").array() {
		let item_node = item.as_node();
		let title = item_node.select("a").first().attr("title").read();
		if title.contains("Light Novel") { continue; }
		let id = item_node.select("a").first().attr("href").read();
		let cover = get_image_src(item_node);
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
			viewer: MangaViewer::Default
		});
	}
	let last_page_string = html.select(".hpage a.r").text().read();
	let has_more;
	if last_page_string.contains("Next") || last_page_string.contains("Suivant") {
		has_more = true;
	} else {
		has_more = false;
	}
	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

// parse the listing page (popular, latest , new etc)
pub fn parse_manga_listing(base_url: String, traverse_pathname: String, listing: Listing, page: i32) -> Result<MangaPageResult> {
	let url = get_listing_url(base_url, traverse_pathname, listing, page);
	let mut mangas: Vec<Manga> = Vec::new();
	let html = Request::new(url.clone().as_str(), HttpMethod::Get).html();
	for manga in html.select(".listupd .bsx").array() {
		let manga_node = manga.as_node();
		let title = manga_node.select("a").attr("title").read();
		if title.contains("Light Novel") { continue; }
		let id = manga_node.select("a").attr("href").read();
		let cover = get_image_src(manga_node);
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
	let last_page_string = html.select(".hpage a.r").text().read();
	let has_more;
	if last_page_string.contains("Next") || last_page_string.contains("Suivant") {
		has_more = true;
	} else {
		has_more = false;
	}
	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

// parse manga details page 
pub fn parse_manga_details(id: String) -> Result<Manga> {
	let html = Request::new(id.clone().as_str(), HttpMethod::Get).html();
	let raw_title = html.select("h1.entry-title").text().read();
	let title: String;
	if raw_title.contains("Read"){
		title = raw_title.replace("Read ", "").replace("English", "");
	} else {
		title = raw_title;
	}
	let image = html.select(".thumb img").attr("src").read();
	let cover: String;
	if image.starts_with("data:") || image == "" {
		cover = html.select(".thumb img").first().attr("data-lazy-src").read();
	} else {
		cover = image;
	}
	let mut author = String::from(html.select("span:contains(Author:), span:contains(Pengarang:), .fmed b:contains(Author)+span, .imptdt:contains(Author) i, .fmed b:contains(Yazar)+span, .fmed b:contains(Autheur)+span").text().read()
		.replace("[Add, ]","")
		.replace("Author","").trim());
	if author == "-" {
        author = String::from("No Author");
    }
	let artist = html.select("#last_episode small").text().read();
	let description = html.select(".entry-content p").text().read();
	let status = manga_status(String::from(html.select(".imptdt:contains(Status), .imptdt:contains(Durum), .imptdt:contains(Statut) i").text().read()
		.replace("Status","")
		.replace("Statut","").to_uppercase().trim()));
	let mut categories = Vec::new();
	let mut nsfw = MangaContentRating::Safe;
	for node in html.select("span.mgen a").array() {
		let category = node.as_node().text().read();
		match nsfw {
			MangaContentRating::Safe => {
				if category.clone().as_str() == "Ecchi" {
					nsfw = MangaContentRating::Suggestive;
				} else if category.clone().as_str() == "Mature" || category.clone().as_str() == "Hentai" || category.clone().as_str() == "Smut" {
					nsfw = MangaContentRating::Nsfw;
				}
			},
			MangaContentRating::Suggestive => {
				if category.clone().as_str() == "Mature" || category.clone().as_str() == "Hentai" || category.clone().as_str() == "Smut" {
					nsfw = MangaContentRating::Nsfw;
				}
			}
    		MangaContentRating::Nsfw => (),
		}
		categories.push(category.clone());
	}
	let manga_type = html.select(".imptdt a").text().read();
	let viewer;
	match manga_type.clone().trim() {
		"Manhwa" | "Manhua"=> viewer = MangaViewer::Scroll,
		// "Korean" => viewer = MangaViewer::Scroll,
		_ => viewer = MangaViewer::Default,
	}
	Ok(Manga {
		id: id.clone(),
		cover,
		title,
		author,
		artist,
		description,
		url: id.clone(),
		categories,
		status,
		nsfw,
		viewer
	})
}


// parse the chapters list present on manga details page
pub fn parse_chapter_list(id: String, date_format: String, language_code: String, locale: &str) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	let html = Request::new(id.clone().as_str(), HttpMethod::Get).html();
	for chapter in html.select("#chapterlist li").array() {
		let chapter_node = chapter.as_node();
		let title = chapter_node.select("span.chapternum").text().read();
		let chapter_id = chapter_node.select("a").attr("href").read();
		let chapter_url = chapter_node.select("a").attr("href").read();
		let chapter_number = get_chapter_number(title.clone());
		let date_updated = get_date(id.clone(), date_format.clone(), locale,chapter_node.select("span.chapterdate").text());
		chapters.push(Chapter {
			id: chapter_id,
			title: String::new(),
			volume: -1.0,
			chapter: chapter_number,
			date_updated,
			scanlator: String::new(),
			url: chapter_url,
			lang: language_code.clone(),
		});
	}
	Ok(chapters)
}

//parse the maga chapter images list
pub fn parse_page_list(id: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	let html = Request::new(&id, HttpMethod::Get).html();
	let mut at = 0;
	for page in html.select("div#readerarea img").array() {
		let page_node = page.as_node();
		let page_url = page_node.attr("src").read();
		pages.push(Page {
			index: at,
			url: page_url,
			base64: String::new(),
			text: String::new(),
		});
		at += 1;
	}
	if !pages.is_empty() {
		Ok(pages)
	} else {
		let raw_text = html.select("script").html().read();
		let trimmed_json = &raw_text[raw_text.find(r#":[{"s"#).unwrap_or(0)+2..raw_text.rfind("}],").unwrap_or(0)+1];
		let trimmed_text: &str;
		if trimmed_json.contains("Default 2") {
			trimmed_text = &trimmed_json[..trimmed_json.rfind(r#",{"s"#).unwrap_or(0)];
		} else {
			trimmed_text = trimmed_json;
		}

		let json = parse(trimmed_text.as_bytes()).as_object()?;
		let images = json.get("images").as_array()?;
		let mut index = 0;
		for page in images {
			let page_url = page.as_string()?.read();
			pages.push(Page { 
				index, 
				url: page_url, 
				base64: String::new(), 
				text: String::new(), 
			});
			index += 1;
		}
		Ok(pages)
	}
}

pub fn modify_image_request(base_url: String, request: Request) {
	request.header("Referer", &base_url);
}

pub fn handle_url(url: String) -> Result<DeepLink> {
	Ok(DeepLink {
		manga: Some(parse_manga_details(url.clone())?),
		chapter: None
	})
}
