use aidoku::{
	error::Result,
	helpers::uri::QueryParameters,
	prelude::format,
	std::{
		html::Node,
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, Filter, FilterType, Manga, MangaPageResult, MangaStatus, MangaViewer, Page,
};

extern crate alloc;
use alloc::string::ToString;

const BASE_URL: &str = "https://www.colamanhua.com";

const FILTER_GENRE: [&str; 31] = [
	"", "10023", "10024", "10126", "10124", "10210", "10143", "10129", "10242", "10560", "10122",
	"10641", "10201", "10138", "10461", "10943", "10301", "10321", "10309", "10125", "10131",
	"10133", "10127", "10142", "10722", "10480", "10706", "11062", "10227", "10183", "10181",
];
const FILTER_STATUS: [&str; 3] = ["", "1", "2"];
const FILTER_ALPHABET: [&str; 27] = [
	"", "10182", "10081", "10134", "10001", "10238", "10161", "10225", "10137", "10284", "10141",
	"10283", "10132", "10136", "10130", "10282", "10262", "10164", "10240", "10121", "10123",
	"11184", "11483", "10135", "10061", "10082", "10128",
];

const SORT: [&str; 4] = ["update", "dailyCount", "weeklyCount", "monthlyCount"];

pub fn get_filtered_url(filters: Vec<Filter>, page: i32, url: &mut String) {
	url.push_str(BASE_URL);

	let mut is_searching = false;
	let mut search_str = String::new();

	let mut genre = None;
	let mut status = None;
	let mut alphabet = None;
	let mut sort_by = None;

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					search_str.push_str(&filter_value.read());
					is_searching = true;
				}
			}
			FilterType::Select => {
				let index = filter.value.as_int().unwrap_or(0) as usize;
				match filter.name.as_str() {
					"類型" => genre = (index != 0).then(|| FILTER_GENRE[index]),
					"狀態" => status = (index != 0).then(|| FILTER_STATUS[index]),
					"字母" => alphabet = (index != 0).then(|| FILTER_ALPHABET[index]),
					_ => continue,
				};
			}
			FilterType::Sort => {
				let index = filter
					.value
					.as_object()
					.expect("value object")
					.get("index")
					.as_int()
					.expect("int index") as usize;
				sort_by = (index != 2).then(|| SORT[index]);
			}
			_ => continue,
		}
	}

	let mut query = QueryParameters::new();
	if is_searching {
		query.push("type", Some("1"));
		query.push("searchString", Some(search_str.as_str()));
		query.push(String::from("page"), (page > 1).then(|| page.to_string()));

		url.push_str(format!("/search?{}", query).as_str());
	} else {
		query.push("mainCategoryId", genre);
		query.push("status", status);
		query.push("charCategoryId", alphabet);
		query.push("orderBy", sort_by);
		query.push(String::from("page"), (page > 1).then(|| page.to_string()));

		url.push_str(format!("/show?{}", query).as_str());
	}
}

pub fn request_get(url: &mut String) -> Request {
	Request::new(url.as_str(), HttpMethod::Get).header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_3_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36")
}

pub fn parse_home_page(html: Node, page: i32) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	let mut has_more = false;

	for item in html.select(".fed-list-item").array() {
		let manga_item = item.as_node().expect("manga node");
		let thumbnail = manga_item.select(".fed-list-pics");
		let url = thumbnail.attr("abs:href").read();
		let id = url.split('-').last().unwrap().replace("/", "");
		let cover = thumbnail.attr("data-original").read();
		let title = manga_item.select(".fed-list-title").text().read();

		mangas.push(Manga {
			id,
			cover,
			title,
			url,
			viewer: MangaViewer::Scroll,
			..Default::default()
		});
	}

	if html
		.select(".fed-btns-info:containsOwn(下页)")
		.attr("class")
		.read() == String::from("fed-btns-info fed-rims-info")
	{
		if page < 10 {
			has_more = true;
		}
	}

	html.close();

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_search_page(html: Node, page: i32) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	let mut has_more = false;

	for item in html.select(".fed-deta-info").array() {
		let manga_item = item.as_node().expect("manga node");
		let thumbnail = manga_item.select(".fed-list-pics");
		let url = thumbnail.attr("abs:href").read();
		let id = url.split('-').last().unwrap().replace("/", "");
		let cover = thumbnail.attr("data-original").read();
		let title = manga_item.select("h1").text().read();
		let author = manga_item
			.select("li:contains(作者)")
			.own_text()
			.read()
			.replace("\"", "")
			.trim()
			.to_string();
		let description = manga_item
			.select("li:contains(简介) > div")
			.own_text()
			.read()
			.replace("\"", "")
			.trim()
			.to_string();

		let mut categories: Vec<String> = Vec::new();
		for genre_item in manga_item.select("li:contains(类别) > a").array() {
			let genre = genre_item.as_node().expect("genre node").text().read();
			categories.push(genre);
		}

		let status_str = manga_item
			.select("li:contains(状态)")
			.own_text()
			.read()
			.replace("\"", "")
			.trim()
			.to_string();
		let status = if status_str == String::from("连载中") {
			MangaStatus::Ongoing
		} else if status_str == String::from("已完结") {
			MangaStatus::Completed
		} else {
			MangaStatus::Unknown
		};

		mangas.push(Manga {
			id,
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
		});
	}

	if (page * 10) < html.select("#fed-count").text().read().parse().unwrap() {
		if page < 10 {
			has_more = true;
		}
	}

	html.close();

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn get_manga_details(html: Node, manga_id: String) -> Result<Manga> {
	let info = html.select(".fed-deta-info");
	let cover = info.select(".fed-list-pics").attr("data-original").read();
	let title = info.select("h1").text().read();
	let author = info.select("li:contains(作者) > a").text().read();
	let description = info.select("li:contains(简介) > div").own_text().read();
	let url = format!("{}/manga-{}/", BASE_URL, manga_id);

	let mut categories: Vec<String> = Vec::new();
	for genre_item in info.select("li:contains(类别) > a").array() {
		let genre = genre_item.as_node().expect("genre node").text().read();
		categories.push(genre);
	}

	let status_str = info.select("li:contains(状态) > a").text().read();
	let status = if status_str == String::from("连载中") {
		MangaStatus::Ongoing
	} else if status_str == String::from("已完结") {
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

	todo!();

	Ok(chapters)
}

pub fn get_page_list(html: Node) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	todo!();

	Ok(pages)
}

pub fn parse_deep_link(deep_link: &mut String) -> (Option<String>, Option<String>) {
	let mut manga_id = None;
	let mut chapter_id = None;

	todo!();

	(manga_id, chapter_id)
}
