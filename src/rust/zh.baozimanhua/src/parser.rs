use aidoku::{
	error::Result,
	helpers::{substring::Substring, uri::QueryParameters},
	prelude::format,
	std::{html::Node, net::Request, String, ValueRef, Vec},
	Chapter, Filter, FilterType, Manga, MangaPageResult, MangaStatus, MangaViewer, Page,
};

extern crate alloc;
use alloc::string::ToString;

pub const BASE_URL: &str = "https://www.baozimh.com";
pub const API_URL: &str = "/api/bzmhq/amp_comic_list";
pub const CHAPTER_BASE_URL: &str = "https://www.kukuc.co";
pub const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_3_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36";
const COVER_BASE_URL: &str = "https://static-tw.baozimh.com/cover";

const CLASSIFY_REGION: [&str; 5] = ["all", "cn", "jp", "kr", "en"];
const CLASSIFY_TYPE: [&str; 26] = [
	"all",
	"lianai",
	"chunai",
	"gufeng",
	"yineng",
	"xuanyi",
	"juqing",
	"kehuan",
	"qihuan",
	"xuanhuan",
	"chuanyue",
	"mouxian",
	"tuili",
	"wuxia",
	"gedou",
	"zhanzheng",
	"rexie",
	"gaoxiao",
	"danuzhu",
	"dushi",
	"zongcai",
	"hougong",
	"richang",
	"hanman",
	"shaonian",
	"qita",
];
const CLASSIFY_FILTER: [&str; 9] = [
	"*",
	"ABCD",
	"EFGH",
	"IJKL",
	"MNOP",
	"QRST",
	"UVW",
	"XYZ",
	"0123456789",
];

pub fn get_filtered_url(filters: Vec<Filter>, page: i32) -> String {
	let mut url = String::from(BASE_URL);

	let mut is_searching = false;
	let mut search_str = String::new();

	let mut classify_region: &str = "all";
	let mut classify_type: &str = "all";
	let mut classify_filter: &str = "*";

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					search_str.push_str(filter_value.read().as_str());
					is_searching = true;
				}
			}
			FilterType::Select => {
				let index = filter.value.as_int().unwrap_or(0) as usize;
				match filter.name.as_str() {
					"地區" => classify_region = CLASSIFY_REGION[index],
					"類型" => classify_type = CLASSIFY_TYPE[index],
					"字母" => classify_filter = CLASSIFY_FILTER[index],
					_ => continue,
				};
			}
			_ => continue,
		}
	}

	let mut query = QueryParameters::new();
	if is_searching {
		query.push("q", Some(search_str.as_str()));

		url.push_str(format!("/search?{}", query).as_str());

		return url;
	}
	query.push("type", Some(classify_type));
	query.push("region", Some(classify_region));
	query.push("filter", Some(classify_filter));
	query.push("page", Some(page.to_string().as_str()));
	query.push("limit", Some("20"));
	query.push("language", Some("tw"));

	url.push_str(format!("{}?{}", API_URL, query).as_str());

	url
}

pub fn request_get(url: String) -> Request {
	Request::get(url).header("User-Agent", USER_AGENT)
}

pub fn parse_home_page(json: ValueRef) -> Result<MangaPageResult> {
	let object = json.as_object()?;

	if object.len() == 1 {
		return Ok(MangaPageResult {
			..Default::default()
		});
	}

	let mut mangas: Vec<Manga> = Vec::new();
	for item in object.get("items").as_array()? {
		let manga_item = item.as_object()?;
		let id = manga_item.get("comic_id").as_string()?.read();

		let cover_str = manga_item.get("topic_img").as_string()?.read();
		let cover = format!("{}/{}", COVER_BASE_URL, cover_str);

		let title = manga_item.get("name").as_string()?.read();
		let author = manga_item.get("author").as_string()?.read();
		let url = format!("{}/comic/{}", BASE_URL, id);

		let mut categories: Vec<String> = Vec::new();
		let genre_arr = manga_item.get("type_names").as_array()?;
		for genre_str in genre_arr {
			let genre = genre_str.as_string()?.read();
			if !genre.is_ascii() {
				categories.push(genre);
			}
		}

		mangas.push(Manga {
			id,
			cover,
			title,
			author: author.clone(),
			artist: author,
			url,
			categories,
			viewer: MangaViewer::Scroll,
			..Default::default()
		});
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more: true,
	})
}

pub fn parse_search_page(html: Node) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	for item in html.select(".comics-card").array() {
		let manga_item = item.as_node()?;

		let poster = manga_item.select(".comics-card__poster");

		let url = poster.attr("abs:href").read();
		let id = url.split('/').last().expect("manga id").to_string();
		let cover = format!("{}/{}.jpg", COVER_BASE_URL, id);
		let title = poster.attr("title").read();
		let author = manga_item.select(".tags").text().read();

		let mut categories: Vec<String> = Vec::new();
		for genre_str in poster.select(".tab").array() {
			let genre = genre_str.as_node()?.text().read();
			categories.push(genre);
		}

		mangas.push(Manga {
			id,
			cover,
			title,
			author: author.clone(),
			artist: author,
			url,
			categories,
			viewer: MangaViewer::Scroll,
			..Default::default()
		});
	}

	Ok(MangaPageResult {
		manga: mangas,
		..Default::default()
	})
}

pub fn get_manga_details(html: Node, manga_id: String) -> Result<Manga> {
	let cover = format!(
		"{}/{}.jpg",
		COVER_BASE_URL,
		manga_id.substring_before_last("_").unwrap_or(&manga_id)
	);
	let title = html.select(".comics-detail__title").text().read();
	let author = html.select(".comics-detail__author").text().read();
	let description = html.select(".comics-detail__desc").text().read();
	let url = format!("{}/comic/{}", BASE_URL, manga_id);
	let mut categories: Vec<String> = Vec::new();

	for genre_item in html.select("span.tag").array() {
		let genre = genre_item
			.as_node()?
			.text()
			.read()
			.replace('\"', "")
			.trim()
			.to_string();
		if !genre.is_empty() {
			categories.push(genre);
		}
	}

	let status_str = categories.remove(0);
	let status = if status_str.contains("連載中") {
		MangaStatus::Ongoing
	} else if status_str.contains("已完結") {
		MangaStatus::Completed
	} else {
		MangaStatus::Unknown
	};

	Ok(Manga {
		id: manga_id,
		cover,
		title,
		author: author.clone(),
		artist: author,
		description,
		url,
		categories,
		status,
		viewer: MangaViewer::Scroll,
		..Default::default()
	})
}

pub fn get_chapter_list(html: Node, manga_id: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	let mut new = true;
	let mut flag = 0.0;
	let mut index = 0;

	for item in html.select(".comics-chapters__item").array() {
		let chapter_item = item.as_node()?;

		let title = chapter_item.text().read();
		let chapter_id = chapter_item
			.attr("href")
			.read()
			.split('=')
			.last()
			.expect("chapter id str")
			.parse::<f32>()
			.expect("chapter id f32");
		let url = format!(
			"{}/comic/chapter/{}/0_{}.html",
			CHAPTER_BASE_URL, manga_id, chapter_id
		);

		let chapter = Chapter {
			id: chapter_id.to_string(),
			title,
			chapter: chapter_id + 1.0,
			url,
			lang: String::from("zh"),
			..Default::default()
		};

		if chapter_id == 0.0 {
			new = false;
		} else if new {
			flag = chapter_id;
		}

		chapters.insert(index, chapter);

		if chapter_id == (flag - 1.0) {
			break;
		}
		if new {
			index += 1;
		}
	}

	Ok(chapters)
}

pub fn get_page_list(url: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	let current_chapter = format!("/0_{}_", chapter_id);
	let mut page_url = url;
	loop {
		let html = request_get(page_url).html()?;

		for (index, item) in html.select(".comic-contain__item").array().enumerate() {
			let url = item.as_node()?.attr("src").read();

			pages.push(Page {
				index: index as i32,
				url,
				..Default::default()
			});
		}

		page_url = html.select("#next-chapter").attr("href").read();
		if !page_url.contains(current_chapter.as_str()) {
			break;
		}
	}

	Ok(pages)
}

pub fn parse_deep_link(deep_link: String) -> (String, String) {
	let mut manga_id = String::new();
	let mut chapter_id = String::new();

	if deep_link.contains("baozimh.com/comic/") {
		if deep_link.contains("/chapter/") {
			let id: Vec<&str> = deep_link.rsplit('/').collect();
			manga_id = id[1].to_string();
			chapter_id = id[0].replace("0_", "").replace(".html", "");
		} else {
			manga_id = deep_link.split('/').last().unwrap_or("").to_string();
		}
	}

	(manga_id, chapter_id)
}
