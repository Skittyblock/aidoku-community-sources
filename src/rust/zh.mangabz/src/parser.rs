use aidoku::{
	error::Result,
	helpers::uri::QueryParameters,
	prelude::{format, println},
	std::{html::Node, net::Request, String, Vec},
	Chapter, Filter, FilterType, Manga, MangaPageResult, MangaStatus,
};

extern crate alloc;
use alloc::string::ToString;

pub const BASE_URL: &str = "https://mangabz.com/";
pub const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_3_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36";
const GENRE: [u8; 10] = [0, 31, 26, 1, 2, 25, 11, 17, 15, 34];
const SORT: [u8; 2] = [10, 2];

pub fn get_filtered_url(filters: Vec<Filter>, page: i32) -> String {
	let mut url = String::from(BASE_URL);

	let mut is_searching = false;
	let mut search_str = String::new();

	let mut genre = 0;
	let mut status = 0;
	let mut sort = 10;

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					is_searching = true;
					search_str = filter_value.read();
				}
			}
			FilterType::Select => {
				let index = filter.value.as_int().unwrap_or(0) as usize;
				match filter.name.as_str() {
					"題材" => genre = GENRE[index],
					"狀態" => status = index as u8,
					_ => continue,
				}
			}
			FilterType::Sort => {
				let index = filter.value.as_int().unwrap_or(0) as usize;
				sort = SORT[index];
			}
			_ => continue,
		}
	}

	if is_searching {
		let mut query = QueryParameters::new();
		query.push("title", Some(search_str.as_str()));
		query.push("page", Some(page.to_string().as_str()));

		url.push_str(format!("search?{}", query).as_str());
	} else {
		url.push_str(format!("manga-list-{}-{}-{}-p{}/", genre, status, sort, page).as_str());
	}

	url
}

pub fn request_get(url: String) -> Request {
	Request::get(url)
		.header("Referer", BASE_URL)
		.header("User-Agent", USER_AGENT)
}

pub fn get_manga_list(html: Node) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	for item in html.select("div.mh-item").array() {
		let manga_item = item.as_node()?;
		let title_node = manga_item.select("h2.title > a");

		let id = title_node
			.attr("href")
			.read()
			.replace('/', "")
			.replace("bz", "");
		let cover = manga_item.select("img.mh-cover").attr("src").read();
		let title = title_node.attr("title").read();
		let url = format!("{}{}bz/", BASE_URL, id);

		let status_str = manga_item.select("span").text().read();
		let status = match status_str.as_str() {
			"最新" => MangaStatus::Ongoing,
			"完結" => MangaStatus::Completed,
			_ => MangaStatus::Unknown,
		};

		mangas.push(Manga {
			id,
			cover,
			title,
			url,
			status,
			..Default::default()
		});
	}

	let has_more = !html
		.select("div.page-pagination a:contains(>)")
		.array()
		.is_empty();

	html.close();

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn get_manga_details(html: Node, id: String) -> Result<Manga> {
	let manga_info = html.select("p.detail-info-tip");

	let cover = html.select("img.detail-info-cover").attr("src").read();
	let title = html.select("p.detail-info-title").text().read();

	let mut artists: Vec<String> = Vec::new();
	for item in manga_info.select("span:contains(作者) > a").array() {
		let artist_str = item.as_node()?.text().read();
		artists.push(artist_str);
	}
	let artist = artists.join("、");

	let description = html.select("p.detail-info-content").text().read();
	let url = format!("{}{}bz/", BASE_URL, id);

	let status_str = manga_info
		.select("span:contains(狀態) > span")
		.text()
		.read();
	let status = match status_str.as_str() {
		"連載中" => MangaStatus::Ongoing,
		"已完結" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};

	let mut categories: Vec<String> = Vec::new();
	for item in manga_info.select("span.item").array() {
		let genre = item.as_node()?.text().read();
		categories.push(genre);
	}

	Ok(Manga {
		id,
		cover,
		title,
		author: artist.clone(),
		artist,
		description,
		url,
		categories,
		status,
		..Default::default()
	})
}

pub fn get_chapter_list(html: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	for (index, item) in html
		.select("a.detail-list-form-item")
		.array()
		.rev()
		.enumerate()
	{
		let chapter_item = item.as_node()?;

		let id = chapter_item.attr("href").read().replace(['/', 'm'], "");
		let title = chapter_item.own_text().read();
		let chapter = (index + 1) as f32;
		let url = format!("{}m{}/", BASE_URL, id);

		chapters.insert(
			0,
			Chapter {
				id,
				title,
				chapter,
				url,
				lang: String::from("zh"),
				..Default::default()
			},
		);
	}

	Ok(chapters)
}
