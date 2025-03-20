use aidoku::{
	error::Result,
	prelude::*,
	std::{
		current_date,
		html::Node,
		net::{HttpMethod, Request},
		String, StringRef, Vec,
	},
	Chapter, DeepLink, Filter, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

use crate::helper::*;

extern crate alloc;
use alloc::string::ToString;

pub struct MadaraSiteData {
	pub base_url: String,
	pub lang: String,

	pub source_path: String,
	pub search_path: String,

	pub search_cookies: String,
	pub post_type: String,

	pub search_selector: String,
	pub image_selector: String,
	pub genre_selector: String,
	pub author_selector: String,
	pub description_selector: String,
	pub chapter_selector: String,
	pub base_id_selector: String,

	pub date_format: String,

	pub status_filter_ongoing: String,
	pub status_filter_completed: String,
	pub status_filter_cancelled: String,
	pub status_filter_on_hold: String,
	pub adult_string: String,
	pub genre_condition: String,
	pub popular: String,
	pub trending: String,

	pub alt_ajax: bool,
	pub user_agent: Option<String>,
	pub use_ajax_listing: bool,

	pub get_manga_id: fn(String, String, String, Option<String>) -> String,
	pub viewer: fn(&Node, &Vec<String>) -> MangaViewer,
	pub status: fn(&Node) -> MangaStatus,
	pub nsfw: fn(&Node, &Vec<String>) -> MangaContentRating,

	pub ignore_class: String,
}

impl Default for MadaraSiteData {
	fn default() -> MadaraSiteData {
		MadaraSiteData {
			base_url: String::new(),
			lang: String::from("en"),
			// www.example.com/{source_path}/manga-id/
			source_path: String::from("manga"),
			// www.example.com/{search_path}/?search_query
			search_path: String::from("page"),
			// selector div for search results page
			search_selector: String::from("div.c-tabs-item__content"),
			// cookies to pass for search request
			search_cookies: String::from("wpmanga-adault=1"),
			// the type of request to perform "post_type={post_type}", some sites (toonily) do not
			// work with the default
			post_type: String::from("wp-manga"),
			// p to select description from
			description_selector: String::from("div.description-summary div p"),
			// a to select author from
			author_selector: String::from("div.author-content a"),
			// selector for chapter list
			chapter_selector: String::from("li.wp-manga-chapter"),
			// a to get the base id from requests to admin-ajax.php
			base_id_selector: String::from("h3.h5 > a"),
			// chapter date format
			date_format: String::from("MMM d, yyyy"),
			// div to select images from a chapter
			image_selector: String::from("div.page-break > img"),
			// div to select all the genres
			genre_selector: String::from("div.genres-content > a"),
			// choose between two options for chapter list POST request
			alt_ajax: false,
			// user agent for all http requests
			user_agent: None,
			// use admin-ajax to get listings
			use_ajax_listing: true,
			// get the manga id from script tag
			get_manga_id: get_int_manga_id,
			// default viewer
			viewer: |html, categories| {
				let series_type = html
					.select("div.post-content_item:contains(Type) div.summary-content")
					.text()
					.read()
					.to_lowercase();

				let webtoon_tags = [
					"manhwa", "manhua", "webtoon", "vertical", "korean", "chinese",
				];
				let rtl_tags = ["manga", "japan"];

				if !series_type.is_empty() {
					for tag in webtoon_tags {
						if series_type.contains(tag) {
							return MangaViewer::Scroll;
						}
					}

					for tag in rtl_tags {
						if series_type.contains(tag) {
							return MangaViewer::Rtl;
						}
					}

					MangaViewer::Scroll
				} else {
					for tag in webtoon_tags {
						if categories.iter().any(|v| v.to_lowercase() == tag) {
							return MangaViewer::Scroll;
						}
					}

					for tag in rtl_tags {
						if categories.iter().any(|v| v.to_lowercase() == tag) {
							return MangaViewer::Rtl;
						}
					}

					MangaViewer::Scroll
				}
			},
			status: |html| {
				let status_str = html
					.select("div.post-content_item:contains(Status) div.summary-content")
					.text()
					.read()
					.to_lowercase();
				match status_str.as_str() {
					"ongoing" => MangaStatus::Ongoing,
					"releasing" => MangaStatus::Ongoing,
					"completed" => MangaStatus::Completed,
					"canceled" => MangaStatus::Cancelled,
					"dropped" => MangaStatus::Cancelled,
					"hiatus" => MangaStatus::Hiatus,
					"on hold" => MangaStatus::Hiatus,
					_ => MangaStatus::Unknown,
				}
			},
			nsfw: |html, categories| {
				if !html
					.select(".manga-title-badges.adult")
					.text()
					.read()
					.is_empty()
				{
					MangaContentRating::Nsfw
				} else {
					let nsfw_tags = ["adult", "mature"];
					let suggestive_tags = ["ecchi"];

					for tag in nsfw_tags {
						if categories.iter().any(|v| v.to_lowercase() == tag) {
							return MangaContentRating::Nsfw;
						}
					}

					for tag in suggestive_tags {
						if categories.iter().any(|v| v.to_lowercase() == tag) {
							return MangaContentRating::Suggestive;
						}
					}

					MangaContentRating::Safe
				}
			},
			// Ignore MangaPageResult manga with this class from a listing. Usually used for novels.
			ignore_class: String::from(".web-novel"),
			// Localization stuff
			status_filter_ongoing: String::from("Ongoing"),
			status_filter_completed: String::from("Completed"),
			status_filter_cancelled: String::from("Cancelled"),
			status_filter_on_hold: String::from("On Hold"),
			adult_string: String::from("Adult Content"),
			genre_condition: String::from("Genre Condition"),
			popular: String::from("Popular"),
			trending: String::from("Trending"),
		}
	}
}

pub fn get_manga_list(
	filters: Vec<Filter>,
	page: i32,
	data: MadaraSiteData,
) -> Result<MangaPageResult> {
	let (url, did_search) = get_filtered_url(filters, page, &data);

	if did_search {
		get_search_result(data, url)
	} else {
		get_series_page(data, "_latest_update", page)
	}
}

pub fn get_search_result(data: MadaraSiteData, url: String) -> Result<MangaPageResult> {
	let mut req = Request::get(&url).header("Cookie", &data.search_cookies);

	req = add_user_agent_header(req, &data.user_agent);

	let html = req.html()?;
	let mut manga: Vec<Manga> = Vec::new();
	let mut has_more = false;

	for item in html.select(data.search_selector.as_str()).array() {
		let obj = item.as_node().expect("node array");

		let id = obj
			.select("a")
			.attr("href")
			.read()
			.replace(&data.base_url, "")
			.replace(&data.source_path, "")
			.replace('/', "");
		let title = obj.select("a").attr("title").read();

		let cover = get_image_url(obj.select("img"));

		let genres = obj.select("div.post-content_item div.summary-content a");
		if genres.text().read().to_lowercase().contains("novel") {
			continue;
		}

		manga.push(Manga {
			id,
			cover,
			title,
			..Default::default()
		});
		has_more = true;
	}

	Ok(MangaPageResult { manga, has_more })
}

pub fn get_series_page(data: MadaraSiteData, listing: &str, page: i32) -> Result<MangaPageResult> {
	// Monkeypatch for now until the source api rewrite
	if !data.use_ajax_listing {
		let listing = match listing {
			"_wp_manga_views" => "views",
			"_wp_manga_week_views_value" => "trending",
			"_latest_update" => "latest",
			_ => "latest",
		};

		let url = format!(
			"{}/page/{}?s&post_type=wp-manga&m_orderby={}",
			data.base_url, page, listing
		);

		return get_search_result(data, url);
	}

	let url = data.base_url.clone() + "/wp-admin/admin-ajax.php";

	let body_content =  format!("action=madara_load_more&page={}&template=madara-core%2Fcontent%2Fcontent-archive&vars%5Bpaged%5D=1&vars%5Borderby%5D=meta_value_num&vars%5Btemplate%5D=archive&vars%5Bsidebar%5D=full&vars%5Bpost_type%5D=wp-manga&vars%5Bpost_status%5D=publish&vars%5Bmeta_key%5D={}&vars%5Border%5D=desc&vars%5Bmeta_query%5D%5Brelation%5D=OR&vars%5Bmanga_archives_item_layout%5D=big_thumbnail", &page-1, listing);

	let mut req = Request::new(url.as_str(), HttpMethod::Post)
		.body(body_content.as_bytes())
		.header("Referer", &data.base_url)
		.header("Content-Type", "application/x-www-form-urlencoded");

	req = add_user_agent_header(req, &data.user_agent);

	let html = req.html()?;

	let mut manga: Vec<Manga> = Vec::new();
	let mut has_more = false;
	for item in html.select("div.page-item-detail").array() {
		let obj = item.as_node().expect("node array");

		if !obj.select(&data.ignore_class).text().read().is_empty() {
			continue;
		}

		let base_id = obj.select(&data.base_id_selector).attr("href").read();
		let final_path = strip_prefix(
			strip_prefix(
				strip_prefix(&base_id, &data.base_url),
				&format!("/{}", data.source_path),
			),
			"/",
		);
		let id = final_path
			.strip_suffix('/')
			.unwrap_or(final_path)
			.to_string();

		// These are useless badges that are added to the title like "HOT", "NEW", etc.
		let title_badges = obj.select("span.manga-title-badges").text().read();
		let mut title = obj.select(&data.base_id_selector).text().read();
		if title.contains(&title_badges) {
			title = title.replace(&title_badges, "");
			title = String::from(title.trim());
		}

		let cover = get_image_url(obj.select("img"));

		manga.push(Manga {
			id,
			cover,
			title,
			..Default::default()
		});
		has_more = true;
	}

	Ok(MangaPageResult { manga, has_more })
}

pub fn get_manga_listing(
	data: MadaraSiteData,
	listing: Listing,
	page: i32,
) -> Result<MangaPageResult> {
	if listing.name == data.popular {
		return get_series_page(data, "_wp_manga_views", page);
	}
	if listing.name == data.trending {
		return get_series_page(data, "_wp_manga_week_views_value", page);
	}
	get_series_page(data, "_latest_update", page)
}

pub fn get_manga_details(manga_id: String, data: MadaraSiteData) -> Result<Manga> {
	let url = if manga_id.starts_with("http") {
		manga_id.clone()
	} else {
		format!("{}/{}/{manga_id}", data.base_url, data.source_path)
	};

	let mut req = Request::get(&url);

	req = add_user_agent_header(req, &data.user_agent);

	let html = req.html()?;

	// These are useless badges that are added to the title like "HOT", "NEW", etc.
	let title_badges = html.select("span.manga-title-badges").text().read();
	let mut title = html.select("div.post-title h1").text().read();
	if title.contains(&title_badges) {
		title = title.replace(&title_badges, "");
		title = String::from(title.trim());
	}
	let cover = get_image_url(html.select("div.summary_image img"));
	let author = html.select(&data.author_selector).text().read();
	let artist = html.select("div.artist-content a").text().read();
	let description = html.select(&data.description_selector).text().read();

	let mut categories: Vec<String> = Vec::new();
	for item in html.select(data.genre_selector.as_str()).array() {
		categories.push(item.as_node().expect("node array").text().read());
	}

	let status = (data.status)(&html);
	let viewer = (data.viewer)(&html, &categories);
	let nsfw = (data.nsfw)(&html, &categories);

	Ok(Manga {
		id: manga_id,
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

pub fn get_chapter_list(manga_id: String, data: MadaraSiteData) -> Result<Vec<Chapter>> {
	let mut url = data.base_url.clone() + "/wp-admin/admin-ajax.php";
	if data.alt_ajax {
		url = data.base_url.clone()
			+ "/" + data.source_path.as_str()
			+ "/" + manga_id.as_str()
			+ "/ajax/chapters";
	}

	let int_id = (data.get_manga_id)(
		manga_id,
		data.base_url.clone(),
		data.source_path.clone(),
		data.user_agent.clone(),
	);
	let body_content = format!("action=manga_get_chapters&manga={}", int_id);

	let mut req = Request::new(url.as_str(), HttpMethod::Post)
		.body(body_content.as_bytes())
		.header("Referer", &data.base_url)
		.header("Content-Type", "application/x-www-form-urlencoded");

	req = add_user_agent_header(req, &data.user_agent);

	let html = req.html()?;

	let mut chapters: Vec<Chapter> = Vec::new();
	for item in html.select(&data.chapter_selector).array() {
		let obj = item.as_node().expect("node array");

		let id = obj
			.select("a")
			.attr("href")
			.read()
			.replace(&(data.base_url.clone() + "/"), "")
			.replace(&(data.source_path.clone() + "/"), "");

		let mut title = String::new();
		let t_tag = obj.select("a").text().read();
		if t_tag.contains('-') {
			title.push_str(t_tag[t_tag.find('-').unwrap() + 1..].trim());
		}

		/*  Chapter number is first occourance of a number in the last element of url
			when split with "/"
			e.g.
			one-piece-color-jk-english/volume-20-showdown-at-alubarna/chapter-177-30-million-vs-81-million/
			will return 177
			parasite-chromatique-french/volume-10/chapitre-062-5/
			will return 62.5
		*/

		let slash_vec = id.as_str().split('/').collect::<Vec<&str>>();

		let dash_split = slash_vec[slash_vec.len() - 2].split('-');
		let dash_vec = dash_split.collect::<Vec<&str>>();

		let mut is_decimal = false;
		let mut chapter = 0.0;
		for obj in dash_vec {
			let mut item = {
				let mut obj = obj;
				if obj.contains('_') {
					obj = obj.split('_').next().unwrap_or(obj);
				}
				obj.replace('/', "").parse::<f32>().unwrap_or(-1.0)
			};
			if item == -1.0 {
				item = String::from(obj.chars().next().unwrap())
					.parse::<f32>()
					.unwrap_or(-1.0);
			}
			if item != -1.0 {
				if is_decimal {
					chapter += item / 10.0;
					break;
				} else {
					chapter = item;
					is_decimal = true;
				}
			}
		}

		let date_str = obj.select("span.chapter-release-date > i").text().read();
		let mut date_updated = StringRef::from(&date_str)
			.0
			.as_date(data.date_format.as_str(), Some("en"), None)
			.unwrap_or(-1.0);
		if date_updated < -1.0 {
			date_updated = StringRef::from(&date_str)
				.0
				.as_date("MMM d, yy", Some("en"), None)
				.unwrap_or(-1.0);
		}
		if date_updated == -1.0 {
			date_updated = current_date();
		}

		let url = obj.select("a").attr("href").read();
		let lang = data.lang.clone();

		chapters.push(Chapter {
			id,
			title,
			volume: -1.0,
			chapter,
			date_updated,
			scanlator: String::new(),
			url,
			lang,
		});
	}
	Ok(chapters)
}

pub fn get_page_list(chapter_id: String, data: MadaraSiteData) -> Result<Vec<Page>> {
	let url = data.base_url.clone() + "/" + data.source_path.as_str() + "/" + chapter_id.as_str();
	let mut req = Request::new(url.as_str(), HttpMethod::Get);

	req = add_user_agent_header(req, &data.user_agent);

	let html = req.html()?;

	let mut pages: Vec<Page> = Vec::new();
	for (index, item) in html
		.select(data.image_selector.as_str())
		.array()
		.enumerate()
	{
		pages.push(Page {
			index: index as i32,
			url: get_image_url(item.as_node().expect("node array")),
			..Default::default()
		});
	}
	Ok(pages)
}

pub fn modify_image_request(base_url: String, request: Request) {
	request.header("Referer", &base_url);
}

pub fn handle_url(url: String, data: MadaraSiteData) -> Result<DeepLink> {
	let mut manga_id = String::new();
	let parse_url = url.as_str().split('/').collect::<Vec<&str>>();
	if parse_url.len() >= 4 {
		manga_id.push_str(parse_url[4]);
	}
	Ok(DeepLink {
		manga: Some(get_manga_details(manga_id, data)?),
		chapter: None,
	})
}
