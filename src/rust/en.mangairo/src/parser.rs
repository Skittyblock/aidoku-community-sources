use aidoku::{
	error::Result, helpers::uri::QueryParameters, prelude::*, std::html::Node, std::String,
	std::Vec, Chapter, Filter, FilterType, Manga, MangaContentRating, MangaStatus, MangaViewer,
	Page,
};
extern crate alloc;
use alloc::string::ToString;

pub const BASE_URL: &str = "https://w.mangairo.com";
pub const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_3_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36";

pub fn parse_manga_list(html: Node, page: i32) -> (Vec<Manga>, bool) {
	let mut result: Vec<Manga> = Vec::new();
	for page in html.select(".story-item").array() {
		let obj = page.as_node().expect("node array");

		let url = obj.select(".story-name a").attr("href").read();
		let id = parse_incoming_url_manga_id(&url);
		let title = obj.select(".story-name a ").text().read();
		let cover = obj.select(".story-list-img img").attr("src").read();

		if let Some(id) = id {
			if !id.is_empty() && !title.is_empty() && !cover.is_empty() {
				result.push(Manga {
					id,
					cover,
					title,
					..Default::default()
				});
			}
		}
	}

	// Example: 'Total: 38,202 stories'
	let total_str: String = html
		.select(".quantitychapter")
		.text()
		.read()
		.replace("Total: ", "")
		.replace(" stories", "")
		.chars()
		.filter(|&c| c != ',')
		.collect();

	let has_more = total_str
		.parse::<i32>()
		.map_or(false, |value| value > result.len() as i32 * page);
	(result, has_more)
}

pub fn parse_manga_details(html: Node, id: String) -> Result<Manga> {
	let title = html
		.select(".breadcrumbs p span a span")
		.last()
		.text()
		.read();
	let cover = html.select(".avatar").attr("src").read();
	let description = html
		.select("div#story_discription p")
		.text()
		.read()
		.trim()
		.to_string();
	let status_str = html
		.select(".story_info_right li:nth-child(5) a")
		.text()
		.read()
		.to_lowercase();

	let url = format!("{}", &id);

	let author: String = html
		.select(".story_info_right li:nth-child(3) a")
		.array()
		.map(|tag| String::from(tag.as_node().expect("node array").text().read().trim()))
		.collect::<Vec<String>>()
		.join(", ");

	let categories: Vec<String> = html
		.select(".story_info_right .a-h")
		.array()
		.map(|tag| tag.as_node().expect("node array").text().read())
		.collect();

	let status = match status_str.as_str() {
		"ongoing" => MangaStatus::Ongoing,
		"completed" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};

	let nsfw = if categories.contains(&String::from("Pornographic"))
		|| categories.contains(&String::from("Adult"))
		|| categories.contains(&String::from("Smut"))
		|| categories.contains(&String::from("Erotica"))
	{
		MangaContentRating::Nsfw
	} else if categories.contains(&String::from("Ecchi")) {
		MangaContentRating::Suggestive
	} else {
		MangaContentRating::Safe
	};

	let viewer = if categories.contains(&String::from("Manhua"))
		|| categories.contains(&String::from("Manhwa"))
		|| categories.contains(&String::from("Webtoons"))
	{
		MangaViewer::Scroll
	} else {
		MangaViewer::Rtl
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

pub fn get_chapter_list(html: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	for chapter in html.select(".chapter_list ul li a").array() {
		let obj = chapter.as_node().expect("node array");
		let url = obj.attr("href").read();
		let id = parse_incoming_url_chapter_id(&url);

		if let Some(id) = id {
			let chapter = id
				.rsplit_once('-')
				.and_then(|v| v.1.parse::<f32>().ok())
				.unwrap_or(-1.0);
			let lang: String = "en".to_string();

			chapters.push(Chapter {
				id,
				chapter,
				url,
				lang,
				..Default::default()
			});
		}
	}
	Ok(chapters)
}

pub fn get_page_list(html: Node) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	for (i, page) in html.select(".panel-read-story img").array().enumerate() {
		let obj = page.as_node().expect("node array");
		let url = obj.attr("src").read();

		pages.push(Page {
			index: i as i32,
			url,
			..Default::default()
		});
	}
	Ok(pages)
}

pub fn get_filtered_url(filters: Vec<Filter>, page: i32) -> String {
	let mut is_searching = false;

	let mut title_filter = String::new();
	let mut author_filter = String::new();
	let mut sort_filter = -1;
	let mut genre_filter = -1;
	let mut status_filter = -1;
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter) = filter.value.as_string() {
					title_filter = urlencode(filter.read().to_lowercase());
					is_searching = true;
				}
			}
			FilterType::Author => {
				if let Ok(filter) = filter.value.as_string() {
					author_filter = urlencode(filter.read().to_lowercase());
					is_searching = true;
				}
			}
			FilterType::Select => {
				if filter.name.as_str() == "Sort" {
					sort_filter = filter.value.as_int().unwrap_or(-1);
				}
				if filter.name.as_str() == "Genre" {
					genre_filter = filter.value.as_int().unwrap_or(-1)
				}
				if filter.name.as_str() == "Status" {
					status_filter = filter.value.as_int().unwrap_or(-1)
				}
			}
			_ => continue,
		}
	}

	let mut search_string = String::new();
	search_string.push_str(&title_filter);
	if !search_string.is_empty() && !author_filter.is_empty() {
		search_string.push('_');
	}
	search_string.push_str(&author_filter);

	if is_searching {
		let mut query = QueryParameters::new();
		query.set("page", Some(page.to_string().as_str()));

		return format!("{BASE_URL}/list/search/{search_string}?{query}");
	}

	let sort = match sort_filter {
		0 => "latest",
		1 => "newest",
		2 => "topview",
		_ => "latest",
	};

	// Genre
	let ctg = match genre_filter {
		0 => "all", // "All",
		1 => "2",   // "Action",
		2 => "3",   // "Adult",
		3 => "4",   // "Adventure",
		4 => "6",   // "Comedy",
		5 => "7",   // "Cooking",
		6 => "9",   // "Doujinshi",
		7 => "10",  // "Drama",
		8 => "11",  // "Ecchi",
		9 => "48",  // "Erotica",
		10 => "12", // "Fantasy",
		11 => "13", // "Gender bender",
		12 => "14", // "Harem",
		13 => "15", // "Historical",
		14 => "16", // "Horror",
		15 => "45", // "Isekai",
		16 => "17", // "Josei",
		17 => "44", // "Manhua",
		18 => "43", // "Manhwa",
		19 => "19", // "Martial arts",
		20 => "20", // "Mature",
		21 => "21", // "Mecha",
		22 => "22", // "Medical",
		23 => "24", // "Mystery",
		24 => "25", // "One shot",
		25 => "47", // "Pornographic",
		26 => "26", // "Phychological",
		27 => "27", // "Romance",
		28 => "28", // "School life",
		29 => "29", // "Sci fi",
		30 => "30", // "Seinen",
		31 => "31", // "Shoujo",
		32 => "32", // "Shoujo ai",
		33 => "33", // "Shounen",
		34 => "34", // "Shounen ai",
		35 => "35", // "Slice of Life",
		36 => "36", // "Smut",
		37 => "37", // "Sports",
		38 => "38", // "Supernatural",
		39 => "39", // "Tragedy",
		40 => "40", // "Webtoons",
		41 => "41", // "Yaoi",
		42 => "42", // "Yuri"
		_ => "all",
	};

	// State
	let status = match status_filter {
		0 => "all",
		1 => "ongoing",
		2 => "completed",
		_ => "all",
	};

	let page = &page.to_string();
	format!("{BASE_URL}/manga-list/type-{sort}/ctg-{ctg}/state-{status}/page-{page}")
}

pub fn parse_incoming_url_manga_id(url: &str) -> Option<String> {
	// https://chap.mangairo.com/story-pn279847
	// https://chap.mangairo.com/story-pn279847/chapter-52
	let mut parts: Vec<&str> = url.split('/').collect();
	// Manga URL as ID because otherwise we cannot differentiate `w` and `chap`
	// subdomains for manga
	if parts.len() >= 4 {
		parts.truncate(4);
	}
	Some(parts.join("/"))

	// Excludes the base URL from the manga id.
	// if parts.len() >= 3 {
	// 	let manga_id = parts[3];
	// 	return Some(format!("{}", manga_id));
	// }

	// None
}

pub fn parse_incoming_url_chapter_id(url: &str) -> Option<String> {
	// https://chap.mangairo.com/story-pn279847/chapter-52
	let parts: Vec<&str> = url.split('/').collect();
	if parts.len() >= 4 {
		let chapter_id = parts[4];
		return Some(format!("{}", chapter_id));
	}

	None
}

// HELPER FUNCTIONS

pub fn urlencode(string: String) -> String {
	let mut str = string.to_lowercase();

	let match_a = [
		'à', 'á', 'ạ', 'ả', 'ã', 'â', 'ầ', 'ấ', 'ậ', 'ẩ', 'ẫ', 'ă', 'ằ', 'ắ', 'ặ', 'ẳ', 'ẵ',
	];
	let match_e = ['è', 'é', 'ẹ', 'ẻ', 'ẽ', 'ê', 'ề', 'ế', 'ệ', 'ể', 'ễ'];
	let match_i = ['ì', 'í', 'ị', 'ỉ', 'ĩ'];
	let match_o = [
		'ò', 'ó', 'ọ', 'ỏ', 'õ', 'ô', 'ồ', 'ố', 'ộ', 'ổ', 'ỗ', 'ơ', 'ờ', 'ớ', 'ợ', 'ở', 'ỡ',
	];
	let match_u = ['ù', 'ú', 'ụ', 'ủ', 'ũ', 'ư', 'ừ', 'ứ', 'ự', 'ử', 'ữ'];
	let match_y = ['ỳ', 'ý', 'ỵ', 'ỷ', 'ỹ'];
	let match_d = "đ";
	let match_symbols = [
		'!', '@', '%', '^', '*', '(', ')', '+', '=', '<', '>', '?', '/', ',', '.', ':', ';', '\'',
		' ', '"', '&', '#', '[', ']', '~', '-', '$', '|', '_',
	];

	str = str.replace(match_a, "a");
	str = str.replace(match_e, "e");
	str = str.replace(match_i, "i");
	str = str.replace(match_o, "o");
	str = str.replace(match_u, "u");
	str = str.replace(match_y, "y");
	str = str.replace(match_d, "d");
	str = str.replace(match_symbols, "_");
	str = str.replace("__", "_");
	str = str.trim_matches('_').to_string();

	str
}
