use aidoku::{
	prelude::*,
	error::Result, std::String, std::Vec, std::net::Request, std::net::HttpMethod, std::StringRef,std::current_date,
	Listing, Manga, MangaPageResult, Page, MangaStatus, MangaContentRating, MangaViewer, Chapter, DeepLink, Filter,
};

use crate::helper::*;

pub struct MadaraSiteData {
	pub base_url: String,
	pub lang: String,
	pub source_path: String,
	pub search_path: String,
	pub search_selector: String,
	pub image_selector: String,
	pub alt_ajax: bool,
}

impl Default for MadaraSiteData {
	fn default() -> MadaraSiteData {
		MadaraSiteData{
			base_url: String::new(),
			lang: String::from("en"),
			source_path: String::from("manga"), // www.example.com/{source_path}/manga-id/
			search_path: String::from("page"), // www.example.com/{search_path}/?query
			search_selector: String::from("div.c-tabs-item__content"), // selector div for search results page
			image_selector: String::from("div.page-break > img"),      // div to select images from a chapter
			alt_ajax: false,                    // choose between two options for chapter list POST request
		}
	}
}

pub fn get_manga_list(filters: Vec<Filter>, page: i32, data: MadaraSiteData) -> Result<MangaPageResult> {
	let mut url = data.base_url.clone();
	let did_search = get_filtered_url(filters, page, &mut url, data.search_path.clone());

	if did_search {
		get_search_result(data, url)
	} else {
		get_series_page(data, "_latest_update", page)
	}
}

pub fn get_search_result(data: MadaraSiteData, url: String) -> Result<MangaPageResult> {
	let html = Request::new(url.as_str(), HttpMethod::Get).html();
	let mut result: Vec<Manga> = Vec::new();
	let mut r_len = 0;

	for item in html.select(data.search_selector.clone().as_str()).array() {
		let obj   = item.as_node();

		let id	= obj.select("a").attr("href").read().replace(&data.base_url.clone(), "").replace(&data.source_path.clone(), "").replace("/", "");
		let title = obj.select("a").attr("title").read();
		let img   = get_image_url(obj.select("img"));

		let genres = obj.select("div.post-content_item div.summary-content a");
		if genres.text().read().to_lowercase().contains("novel") {
			continue;
		}

		result.push(Manga {
			id,
			cover: img,
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
		r_len += 1;
	}

	Ok(MangaPageResult { manga: result, has_more: r_len > 0, })
}

pub fn get_series_page(data: MadaraSiteData, listing: &str, page: i32) -> Result<MangaPageResult> {
	let url = data.base_url.clone() + "/wp-admin/admin-ajax.php";

	let body_content =  format!("action=madara_load_more&page={}&template=madara-core%2Fcontent%2Fcontent-archive&vars%5Bpaged%5D=1&vars%5Borderby%5D=meta_value_num&vars%5Btemplate%5D=archive&vars%5Bsidebar%5D=full&vars%5Bpost_type%5D=wp-manga&vars%5Bpost_status%5D=publish&vars%5Bmeta_key%5D={}&vars%5Border%5D=desc&vars%5Bmeta_query%5D%5Brelation%5D=OR&vars%5Bmanga_archives_item_layout%5D=big_thumbnail", &page-1, listing);

	let req = Request::new(url.clone().as_str(),  HttpMethod::Post).body(body_content.as_bytes()).header("Content-Type", "application/x-www-form-urlencoded");

	let html = req.html();
	let mut result: Vec<Manga> = Vec::new();

	let mut r_len = 0;
	for item in html.select("div.page-item-detail").array() {
		let obj = item.as_node();

		let w_novel = obj.select(".web-novel").text().read();
		if w_novel.len() > 0 {
			continue;
		}

		let id = obj.select("h3.h5 > a").attr("href").read()
			.replace(&data.base_url.clone(), "")
			.replace(&data.source_path.clone(), "")
			.replace("/", "");

		let title = obj.select("h3.h5 > a").text().read();
		let img = get_image_url(obj.select("img"));

		result.push(Manga {
			id,
			cover: img,
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
		r_len += 1;
	}

	Ok(MangaPageResult {
		manga: result,
		has_more: r_len > 0,
	})
}

pub fn get_manga_listing(data: MadaraSiteData, _listing: Listing, page: i32) -> Result<MangaPageResult> {
	if _listing.name == "Popular" {
		return get_series_page(data, "_wp_manga_views", page);
	}
	if _listing.name == "Trending" {
		return get_series_page(data, "_wp_manga_week_views_value", page);
	}

	let result: Vec<Manga> = Vec::new();
	Ok(MangaPageResult {
		manga: result,
		has_more: false,
	})
}

pub fn get_manga_details(manga_id: String, data: MadaraSiteData) -> Result<Manga> {
	let url = data.base_url.clone() + "/" +
			  data.source_path.clone().as_str()+ "/" +
			  manga_id.as_str();

	let html = Request::new(url.clone().as_str(), HttpMethod::Get).html();

	let title = html.select("div.post-title h1").text().read();

	let img = get_image_url(html.select("div.summary_image img"));
	let author = html.select("div.author-content a").text().read();
	let artist = html.select("div.artist-content a").text().read();
	let description = html.select("div.description-summary div p").text().read();

	let mut categories: Vec<String> = Vec::new();
	for item in html.select("div.genres-content a").array() {
		let obj = item.as_node();
		let category = obj.text().read();
		categories.push(category);
	}

	let mut status = MangaStatus::Unknown;
	let mut viewer = MangaViewer::Default;
	html.select("div.post-content_item")
		.array()
		.for_each(|item| {
			let obj = item.as_node();
			let obj_type = obj.select("h5").text().read();
			if obj_type == "Status" {
				let item_str = obj.select("div.summary-content").text().read().to_lowercase();
				if item_str == "ongoing" {
					status = MangaStatus::Ongoing;
				} else {
					status = MangaStatus::Completed;
				}
			} else {
				status = MangaStatus::Unknown;
			}
			if obj_type == "Type" {
				let item_str = obj.select("div.summary-content").text().read().to_lowercase();
				if item_str.contains("manhwa") {
					viewer = MangaViewer::Scroll;
				}
			}
		});

	let mut nsfw = MangaContentRating::Safe;
	if html.select(".manga-title-badges.adult").text().read().len() > 0 {
		nsfw = MangaContentRating::Nsfw;
	}

	Ok(Manga {
		id: manga_id,
		cover: img,
		title,
		author,
		artist,
		description,
		url,
		categories,
		status,
		nsfw,
		viewer
	})
}

pub fn get_chapter_list(manga_id: String, data: MadaraSiteData) -> Result<Vec<Chapter>> {
	let mut url = data.base_url.clone() + "/wp-admin/admin-ajax.php";
	if data.alt_ajax {
		url = data.base_url.clone() + "/"
		+  data.source_path.clone().as_str()+ "/" +
		manga_id.as_str() + "/"
		+ "ajax/chapters";
	}

	let int_id = get_int_manga_id(manga_id, data.base_url.clone(), data.source_path.clone());
	let body_content =  format!("action=manga_get_chapters&manga={}", int_id);

	let req = Request::new(url.clone().as_str(),  HttpMethod::Post)
		.body(body_content.as_bytes())
		.header("Content-Type", "application/x-www-form-urlencoded");

	let html = req.html();

	let mut chapters: Vec<Chapter> = Vec::new();
	for item in html.select("li.wp-manga-chapter  ").array() {
		let obj = item.as_node();

		let id = obj.select("a").attr("href").read()
			.replace(&(data.base_url.clone() + "/"), "")
			.replace(&(data.source_path.clone() + "/"), "");
		let mut title = String::new();

		let title_html = obj.select("a").text().read();
		if title_html.contains("-") {
			title.push_str(title_html[title_html.find("-").unwrap()+1..].trim());
		}

		/*  Chapter number is first occourance of a number in the last element of url
			when split with "/"
			e.g.
			one-piece-color-jk-english/volume-20-showdown-at-alubarna/chapter-177-30-million-vs-81-million/
			will return 177
			parasite-chromatique-french/volume-10/chapitre-062/
			will return 62
		*/
		let mut chap_num = 0.0;

		let slash_split = id.as_str().split("/");
		let slash_vec = slash_split.collect::<Vec<&str>>();

		let dash_split = slash_vec[slash_vec.len()-2].split("-");
		let dash_vec = dash_split.collect::<Vec<&str>>();

		for obj in dash_vec {
			let item = obj.replace("/", "").parse::<f32>().unwrap_or(-1.0);
			if item != -1.0 {
				chap_num = item;
				break;
			}
		}

		let date_str = obj.select("span.chapter-release-date > i").text().read();
		let mut date = StringRef::from(&date_str).0.as_date("MMM d, yyyy", Some("en"), None).unwrap_or(-1.0);

		if date == -1.0 {
			date = current_date()
		}
		let chap_url = obj.select("a").attr("href").read();

		chapters.push(Chapter {
			id: String::from(id),
			title,
			volume: -1.0,
			chapter: chap_num,
			date_updated: date,
			scanlator: String::new(),
			url: chap_url,
			lang: data.lang.clone(),
		});
	}
	Ok(chapters)
}

pub fn get_page_list(chapter_id: String, data: MadaraSiteData) -> Result<Vec<Page>> {
	let url = data.base_url.clone()  + "/" +
			data.source_path.clone().as_str() + "/" +
			chapter_id.as_str();

	let mut pages: Vec<Page> = Vec::new();
	let html = Request::new(url.clone().as_str(), HttpMethod::Get).html();
	let mut ind = 0;
	for item in html.select(data.image_selector.clone().as_str()).array() {
		let obj = item.as_node();
		let page_url = get_image_url(obj);
		pages.push(Page {
			index: ind,
			url: page_url,
			base64: String::new(),
			text: String::new(),
		});
		ind += 1;
	}
	Ok(pages)
}

pub fn modify_image_request(base_url: String, request: Request) {
	request.header("Referer", &base_url);
}

pub fn handle_url(url: String, data: MadaraSiteData) -> Result<DeepLink> {
	Ok(DeepLink {
		manga: Some(get_manga_details(url.clone(), data)?),
		chapter: None
	})
}
