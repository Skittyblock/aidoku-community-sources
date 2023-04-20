use aidoku::{
	error::Result,
	helpers::{substring::Substring, uri::QueryParameters},
	prelude::format,
	std::{
		html::Node,
		net::{HttpMethod, Request},
		String, ValueRef, Vec,
	},
	Chapter, Filter, FilterType, Manga, MangaPageResult, MangaStatus, MangaViewer, Page,
};

extern crate alloc;
use alloc::string::ToString;

const BASE_URL: &str = "https://www.baozimh.com";

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

pub fn get_filtered_url(filters: Vec<Filter>, page: i32, url: &mut String) {
	url.push_str(BASE_URL);

	let mut is_searching = false;
	let mut search_str = String::new();

	let mut c_region: &str = "all";
	let mut c_type: &str = "all";
	let mut c_filter: &str = "*";

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					search_str.push_str(&filter_value.read().as_str());
					is_searching = true;
				}
			}
			FilterType::Select => {
				let index = filter.value.as_int().unwrap_or(0) as usize;
				match filter.name.as_str() {
					"地區" => c_region = CLASSIFY_REGION[index],
					"類型" => c_type = CLASSIFY_TYPE[index],
					"依字母篩選" => c_filter = CLASSIFY_FILTER[index],
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
	} else {
		query.push("type", Some(c_type));
		query.push("region", Some(c_region));
		query.push("filter", Some(c_filter));
		query.push("page", Some(page.to_string().as_str()));
		query.push("limit", Some("20"));
		query.push("language", Some("tw"));

		url.push_str(format!("/api/bzmhq/amp_comic_list?{}", query).as_str());
	}
}

pub fn request_get(url: &mut String) -> Request {
	Request::new(url.as_str(), HttpMethod::Get).header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_3_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36")
}

pub fn parse_home_page(json_data: ValueRef) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	let mut has_more = false;

	let object = json_data.as_object().expect("json object");
	if object.len() != 1 {
		has_more = true;

		for item in object.get("items").as_array().expect("manga array") {
			let manga_item = item.as_object().expect("manga object");
			let id = manga_item
				.get("comic_id")
				.as_string()
				.expect("id String")
				.read();

			let cover_str = manga_item
				.get("topic_img")
				.as_string()
				.expect("cover String")
				.read();
			let cover = format!("https://static-tw.baozimh.com/cover/{}", cover_str);

			let title = manga_item
				.get("name")
				.as_string()
				.expect("title String")
				.read();
			let author = manga_item
				.get("author")
				.as_string()
				.expect("author String")
				.read();
			let url = format!("{}/comic/{}", BASE_URL, id);

			let mut categories: Vec<String> = Vec::new();
			let genre_arr = manga_item
				.get("type_names")
				.as_array()
				.expect("genre array");
			for genre_str in genre_arr {
				let genre = genre_str.as_string().expect("genre String").read();
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
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_search_page(html: Node) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	for item in html.select(".comics-card").array() {
		let manga_item = item.as_node().expect("manga node");

		let poster = manga_item.select(".comics-card__poster");

		let url = poster.attr("abs:href").read();
		let id = url.split('/').last().unwrap().to_string();
		let cover = format!("https://static-tw.baozimh.com/cover/{}.jpg", id);
		let title = poster.attr("title").read();
		let author = manga_item.select(".tags").text().read();

		let mut categories: Vec<String> = Vec::new();
		for genre_str in poster.select(".tab").array() {
			let genre = genre_str.as_node().expect("genre String").text().read();
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
		"https://static-tw.baozimh.com/cover/{}.jpg",
		manga_id.substring_before_last("_").unwrap_or(&manga_id)
	);
	let title = html.select(".comics-detail__title").text().read();
	let author = html.select(".comics-detail__author").text().read();
	let description = html.select(".comics-detail__desc").text().read();
	let url = format!("{}/comic/{}", BASE_URL, manga_id);
	let mut categories: Vec<String> = Vec::new();

	for genre_item in html.select("span.tag").array() {
		let genre = genre_item
			.as_node()
			.expect("genre node")
			.text()
			.read()
			.replace("\"", "")
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
		let chapter_item = item.as_node().expect("chapter node");

		let title = chapter_item.text().read();
		let chapter_id = chapter_item
			.attr("href")
			.read()
			.split('=')
			.last()
			.unwrap()
			.parse::<f32>()
			.unwrap();
		let url = format!(
			"{}/comic/chapter/{}/0_{}.html",
			BASE_URL, manga_id, chapter_id
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

pub fn get_page_list(html: Node) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	let mut index = 0;
	for item in html.select(".comic-contain__item").array() {
		let url = item.as_node().expect("page url").attr("src").read();

		pages.push(Page {
			index,
			url,
			..Default::default()
		});
		index += 1;
	}

	Ok(pages)
}

pub fn parse_deep_link(deep_link: &mut String) -> (Option<String>, Option<String>) {
	let mut manga_id = None;
	let mut chapter_id = None;

	if deep_link.contains("baozimh.com/comic/") {
		if deep_link.contains("/chapter/") {
			let mut id = deep_link
				.substring_after_last("/chapter/")
				.expect("id &str")
				.split('/');
			manga_id = Some(id.nth(0).unwrap().to_string());
			chapter_id = Some(id.last().unwrap().replace(".html", "").replace("0_", ""));
		} else {
			manga_id = Some(deep_link.split('/').last().unwrap().to_string());
		}
	}

	(manga_id, chapter_id)
}
