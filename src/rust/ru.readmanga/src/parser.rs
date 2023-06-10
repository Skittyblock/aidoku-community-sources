use core::{fmt::Display, iter::once};

use aidoku::{
	error::{AidokuError, Result},
	helpers::{substring::Substring, uri::encode_uri},
	prelude::*,
	std::{
		html::Node,
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

extern crate alloc;
use alloc::{boxed::Box, string::ToString};

use const_format::formatcp;
use itertools::chain;

use crate::wrappers::{debug, WNode};

const BASE_URL: &str = "https://readmanga.live";
const BASE_SEARCH_URL: &str = formatcp!("{}/{}", BASE_URL, "search/advancedResults?");

const SEARCH_OFFSET_STEP: i32 = 50;

#[derive(Debug, Default)]
pub enum Sorting {
	#[default]
	Rating,
	Popular,
	UpdatedRecently,
}

impl Sorting {
	pub fn from_listing(listing: &Listing) -> Self {
		match listing.name.as_str() {
			"Rating" => Self::Rating,
			"Popular" => Self::Popular,
			"Updated Recently" => Self::UpdatedRecently,
			_ => Self::Rating,
		}
	}
}

impl Display for Sorting {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Sorting::Rating => write!(f, "RATING"),
			Sorting::Popular => write!(f, "POPULARITY"),
			Sorting::UpdatedRecently => write!(f, "DATE_UPDATE"),
		}
	}
}

pub fn new_get_request(url: String) -> Result<WNode> {
	Request::new(url, HttpMethod::Get)
		.html()
		.map(WNode::from_node)
}

pub fn create_manga_page_result(mangas: Vec<Manga>) -> MangaPageResult {
	let has_more = !mangas.is_empty();
	MangaPageResult {
		manga: mangas,
		has_more,
	}
}

pub fn parse_directory(html: WNode) -> Result<Vec<Manga>> {
	let nodes = html.select("div.tile");
	// debug!("{:?}", nodes);

	let mangas: Vec<_> = nodes
		.into_iter()
		.filter_map(|node| {
			let div_img_node = node.select("div.img").pop()?;
			// debug!("div_img_node: {div_img_node:?}");

			let id = {
				let a_non_hover_node = div_img_node.select("a.non-hover").pop()?;
				// debug!("a_non_hover_node: {a_non_hover_node:?}");
				a_non_hover_node
					.attr("href")?
					.trim_start_matches('/')
					.to_string()
			};
			debug!("id: {id}");

			let img_node = div_img_node.select("img").pop()?;
			// debug!("img_node: {img_node:?}");
			let cover = img_node.attr("original")?;
			debug!("cover: {cover}");

			let title = img_node.attr("title")?;
			debug!("title: {title}");

			let div_desc_node = node.select("div.desc").pop()?;

			let div_tile_info_node = div_desc_node.select("div.tile-info").pop()?;
			let a_person_link_nodes = div_tile_info_node.select("a.person-link");
			let author = a_person_link_nodes
				.iter()
				.map(WNode::text)
				.intersperse(", ".to_string())
				.collect();
			debug!("author: {author}");

			let div_html_popover_holder_node =
				div_desc_node.select("div.html-popover-holder").pop()?;

			let div_manga_description_node = div_html_popover_holder_node
				.select("div.manga-description")
				.pop()?;
			let description = div_manga_description_node.text();
			debug!("description: {description}");

			let url = format!("{}/{}", BASE_URL, id);
			debug!("url: {}", url);

			let categories = div_html_popover_holder_node
				.select("span.badge-light")
				.iter()
				.map(WNode::text)
				.collect();
			debug!("categories: {categories:?}");

			// TODO: implement more correct status parsing
			let status = {
				if let [span_node] = &node.select("span.mangaTranslationCompleted")[..] {
					if span_node.text() == "переведено" {
						MangaStatus::Completed
					} else {
						MangaStatus::Unknown
					}
				} else if let [_] = &div_img_node.select("div.manga-updated")[..] {
					MangaStatus::Ongoing
				} else {
					MangaStatus::Unknown
				}
			};
			debug!("status: {status:?}");

			Some(Manga {
				id,
				cover,
				title,
				author,
				artist: "".to_string(),
				description,
				url,
				categories,
				status,
				nsfw: MangaContentRating::Suggestive,
				viewer: MangaViewer::Rtl,
			})
		})
		.collect();

	Ok(mangas)
}

pub fn parse_directory_mangafox(html: Node) -> Result<MangaPageResult> {
	let mut result: Vec<Manga> = Vec::new();
	let has_more: bool = !is_last_page_mangafox(html.clone());

	for page in html.select("ul.line li").array() {
		let obj = page.as_node().expect("html array not an array of nodes");

		let id = obj
			.select("a")
			.attr("href")
			.read()
			.replace("/manga/", "")
			.replace('/', "");
		let title = obj.select("a").attr("title").read();
		let cover = obj.select("a img").attr("src").read();

		result.push(Manga {
			id,
			cover,
			title,
			status: MangaStatus::Unknown,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Rtl,
			..Default::default()
		});
	}
	Ok(MangaPageResult {
		manga: result,
		has_more,
	})
}

pub fn parse_manga_mangafox(obj: Node, id: String) -> Result<Manga> {
	let cover = obj.select(".detail-info-cover-img").attr("src").read();
	let title = obj
		.select("span.detail-info-right-title-font")
		.text()
		.read();
	let author = obj.select("p.detail-info-right-say a").text().read();
	let description = obj.select("p.fullcontent").text().read();

	let url = format!("https://www.fanfox.net/manga/{}", &id);

	let mut viewer = MangaViewer::Rtl;
	let mut nsfw: MangaContentRating = MangaContentRating::Safe;
	let mut categories: Vec<String> = Vec::new();
	obj.select(".detail-info-right-tag-list a")
		.array()
		.for_each(|tag_html| {
			let tag = String::from(
				tag_html
					.as_node()
					.expect("Array of tags wasn't nodes.")
					.text()
					.read()
					.trim(),
			);
			if tag == "Ecchi" || tag == "Mature" || tag == "Smut" || tag == "Adult" {
				nsfw = MangaContentRating::Nsfw;
			}
			if tag == "Webtoons" {
				viewer = MangaViewer::Scroll;
			}
			categories.push(tag);
		});

	let status_str = obj
		.select(".detail-info-right-title-tip")
		.text()
		.read()
		.to_lowercase();
	let status = if status_str.contains("Ongoing") {
		MangaStatus::Ongoing
	} else if status_str.contains("Completed") {
		MangaStatus::Completed
	} else {
		MangaStatus::Unknown
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

pub fn parse_chapters_mangafox(obj: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	for item in obj.select(".detail-main-list li").array() {
		let obj = item.as_node().expect("");
		let id = obj
			.select("a")
			.attr("href")
			.read()
			.replace("/manga/", "")
			.replace("/1.html", "");

		let url = format!("https://www.fanfox.net/manga/{}", &id);

		// parse title
		let mut title = String::new();
		let title_str = obj.select(".title3").text().read();
		let split = title_str.as_str().split('-');
		let vec = split.collect::<Vec<&str>>();
		if vec.len() > 1 {
			let (_, rest) = vec.split_first().unwrap();
			title = rest.join("-")
		}

		let mut volume = -1.0;
		let mut chapter = -1.0;

		// parse volume and chapter
		let split = id.as_str().split('/');
		let vec = split.collect::<Vec<&str>>();
		for item in vec {
			let f_char = &item.chars().next().unwrap();
			match f_char {
				'v' => {
					volume = String::from(item)
						.trim_start_matches('v')
						.parse::<f32>()
						.unwrap_or(-1.0)
				}
				'c' => {
					chapter = String::from(item)
						.trim_start_matches('c')
						.parse::<f32>()
						.unwrap_or(-1.0)
				}
				_ => continue,
			}
		}

		let date_updated = obj
			.select(".title2")
			.text()
			.0
			.as_date("MMM dd,yyyy", None, None)
			.unwrap_or(-1.0);

		chapters.push(Chapter {
			id,
			title,
			volume,
			chapter,
			date_updated,
			url,
			lang: String::from("en"),
			..Default::default()
		});
	}
	Ok(chapters)
}

pub fn get_page_list_mangafox(html: Node) -> Result<Vec<Page>> {
	// Unpacker script
	// https://github.com/Skittyblock/aidoku-community-sources/commit/616199e0ccb3704c45438b9f863641e1aa0cfa19
	let mut pages: Vec<Page> = Vec::new();
	for (index, item) in html.select("#viewer img").array().enumerate() {
		let obj = item.as_node().expect("");
		let url = format!(
			"https://{}",
			obj.attr("data-original").read().replace("//", "")
		);
		pages.push(Page {
			index: index as i32,
			url: url.to_string(),
			..Default::default()
		});
	}
	if pages.is_empty() {
		pages.push(Page {
			index: 1,
			url: "https://i.imgur.com/5mNXCgV.png".to_string(),
			..Default::default()
		});
	}

	Ok(pages)
}

pub fn get_filter_url(filters: &Vec<Filter>, sorting: Sorting, page: i32) -> Result<String> {
	fn get_handler(operation: &'static str) -> Box<dyn Fn(AidokuError) -> AidokuError> {
		return Box::new(move |err: AidokuError| {
			println!("Error {:?} while {}", err.reason, operation);
			err
		});
	}

	let filter_parts: Vec<_> = filters
		.iter()
		.filter_map(|filter| match filter.kind {
			FilterType::Title => filter
				.value
				.clone()
				.as_string()
				.map_err(get_handler("casting to string"))
				.ok()
				.map(|title| format!("q={}", encode_uri(title.read()))),
			_ => None,
		})
		.collect();

	let offset = format!("offset={}", (page - 1) * SEARCH_OFFSET_STEP);
	let sort = format!("sortType={}", sorting);

	Ok(format!(
		"{}{}",
		BASE_SEARCH_URL,
		chain!(once(offset), once(sort), filter_parts.into_iter())
			.intersperse("&".to_string())
			.collect::<String>()
	))
}

pub fn get_filtered_url_mangafox(filters: Vec<Filter>, page: i32) -> String {
	let mut is_searching = false;
	let mut search_query = String::new();
	let mut url = String::from("https://fanfox.net");

	let mut genres = String::new();
	let mut nogenres = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					search_query.push_str("&name=");
					search_query.push_str(encode_uri(filter_value.read().to_lowercase()).as_str());
					is_searching = true;
				}
			}
			FilterType::Genre => {
				if let Ok(filter_id) = filter.object.get("id").as_string() {
					match filter.value.as_int().unwrap_or(-1) {
						0 => {
							nogenres.push_str(filter_id.read().as_str());
							nogenres.push(',');
							is_searching = true;
						}
						1 => {
							genres.push_str(filter_id.read().as_str());
							genres.push(',');
							is_searching = true;
						}
						_ => continue,
					}
				}
			}
			FilterType::Select => {
				if filter.name == "Language" {
					search_query.push_str("&type=");
					if filter.value.as_int().unwrap_or(-1) > 0 {
						search_query
							.push_str(&(filter.value.as_int().unwrap_or(-1) as i32).to_string());
						is_searching = true;
					}
				}
				if filter.name == "Rating" {
					search_query.push_str("&rating_method=eq&rating=");
					if filter.value.as_int().unwrap_or(-1) > 0 {
						search_query
							.push_str(&(filter.value.as_int().unwrap_or(-1) as i32).to_string());
						is_searching = true;
					}
				}
				if filter.name == "Completed" {
					search_query.push_str("&st=");
					search_query
						.push_str(&(filter.value.as_int().unwrap_or(-1) as i32).to_string());
					if filter.value.as_int().unwrap_or(-1) > 0 {
						is_searching = true;
					}
				}
			}
			_ => continue,
		}
	}

	if is_searching {
		let search_string = if page == 1 {
			format!(
                "/search?title=&stype=1&author_method=cw&author=&artist_method=cw&artist={}&released_method=eq&released=&genres={}&nogenres={}",
                &search_query,
                &genres.trim_end_matches(','),
                &nogenres.trim_end_matches(','),
            )
		} else {
			format!(
                "/search?page={}&author_method=cw&author=&artist_method=cw&artist={}&genres={}&nogenres={}&released_method=eq&released=&stype=1",
                &page.to_string(),
                &search_query,
                &genres.trim_end_matches(','),
                &nogenres.trim_end_matches(','),
            )
		};

		url.push_str(search_string.as_str());
	} else {
		let list_string = format!("/directory?page={}.html?rating", &page.to_string());
		url.push_str(list_string.as_str());
	}
	encode_uri(url)
}

pub fn parse_incoming_url_mangafox(url: String) -> String {
	// https://fanfox.net/manga/solo_leveling
	// https://fanfox.net/manga/solo_leveling/c183/1.html#ipg2
	// https://m.fanfox.net/manga/chainsaw_man/
	// https://m.fanfox.net/manga/onepunch_man/vTBD/c178/1.html
	let mut manga_id = url
		.substring_after("/manga/")
		.expect("Could not parse the chapter URL. Make sure the URL for MangaFox is correct.");
	if manga_id.contains('/') {
		manga_id = manga_id.substring_before("/").unwrap();
	}
	manga_id.to_string()
}

pub fn is_last_page_mangafox(html: Node) -> bool {
	let length = &html.select("div.pager-list-left a").array().len();
	for (index, page) in html.select("div.pager-list-left a").array().enumerate() {
		let page_node = page.as_node().expect("Failed to get page node");
		let href = page_node.attr("href").read();
		if index == length - 1 && href == "javascript:void(0)" {
			return true;
		}
	}
	false
}
