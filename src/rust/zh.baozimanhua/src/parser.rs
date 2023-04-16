use aidoku::{
	error::Result,
	helpers::uri::encode_uri,
	prelude::{format, println},
	std::{html::Node, String, ValueRef, Vec},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

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
	let mut is_searching = false;
	let mut search_str = String::new();
	url.push_str(BASE_URL);

	let mut c_region: &str = "all";
	let mut c_type: &str = "all";
	let mut c_filter: &str = "*";

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					search_str.push_str(encode_uri(&filter_value.read()).as_str());
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

	if is_searching {
		url.push_str(format!("/search?q={}", search_str).as_str());
	} else {
		url.push_str(
			format!(
				"/api/bzmhq/amp_comic_list?type={}&region={}&filter={}&page={}",
				c_type, c_region, c_filter, page
			)
			.as_str(),
		);
	}
}

pub fn parse_home_page(json_data: ValueRef) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	let mut has_more = false;
	if json_data.clone().as_object()?.len() != 1 {
		has_more = true;

		for item in json_data.as_object()?.get("items").as_array()? {
			let manga_item = item.as_object()?;
			let id = manga_item.get("comic_id").as_string()?.read();
			let cover = format!("https://static-tw.baozimh.com/cover/{}.jpg", id);
			let title = manga_item.get("name").as_string()?.read();
			let author = manga_item.get("author").as_string()?.read();
			let url = format!("{}/comic/{}", BASE_URL, id);
			let categories_array = manga_item.get("type_names").as_array()?;

			let mut categories: Vec<String> = Vec::new();
			for category in categories_array {
				let category_str = category.as_node()?.text().read();
				if !category_str.is_ascii() {
					categories.push(category_str);
				}
			}

			let manga = Manga {
				id,
				cover,
				title,
				author: author.clone(),
				artist: author,
				description: String::new(),
				url,
				categories,
				status: MangaStatus::Unknown,
				nsfw: MangaContentRating::Safe,
				viewer: MangaViewer::Scroll,
			};
			mangas.push(manga);
		}
	}

	let mut test: Vec<Manga> = Vec::new();
	let test_manga = Manga {
		id: String::from("wuliandianfeng-pikapi"),
		cover: String::from("https://static-tw.baozimh.com/cover/wuliandianfeng-pikapi.jpg"),
		title: String::from("武炼巅峰"),
		author: String::from("噼咔噼"),
		artist: String::from("噼咔噼"),
		description: String::new(),
		url: String::from("https://www.baozimh.com/comic/wuliandianfeng-pikapi"),
		categories: Vec::new(),
		status: MangaStatus::Unknown,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Scroll,
	};
	test.push(test_manga);

	Ok(MangaPageResult {
		manga: test,
		has_more: false,
	})
}

pub fn parse_search_page(html: Node) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	for item in html.select(".comics-card").array() {
		let manga_item = item.as_node()?;
		let poster = manga_item.select("a").first();
		let id = String::from(poster.attr("href").read().split('/').last().unwrap());
		let cover = format!("https://static-tw.baozimh.com/cover/{}.jpg", id);
		let title = poster.attr("title").read();
		let author = manga_item.select("a").last().select(".tags").text().read();
		let url = format!("{}/comic/{}", BASE_URL, id);
		let categories_array = poster.select(".tab").array();

		let mut categories: Vec<String> = Vec::new();
		for category in categories_array {
			let category_str = category.as_node()?.text().read();
			categories.push(category_str);
		}

		let manga = Manga {
			id,
			cover,
			title,
			author: author.clone(),
			artist: author,
			description: String::new(),
			url,
			categories,
			status: MangaStatus::Unknown,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll,
		};
		mangas.push(manga);
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more: false,
	})
}

pub fn get_manga_details(html: Node, manga_id: String) -> Result<Manga> {
	let cover = format!("https://static-tw.baozimh.com/cover/{}.jpg", manga_id);
	// let title = html.select(".comics-detail__title").text().read();
	let title = html
		.select("meta[name='og:novel:book_name']")
		.attr("content")
		.read();
	// let author = html.select(".comics-detail__author").text().read();
	let author = html
		.select("meta[name='og:novel:author']")
		.attr("content")
		.read();
	let description = html.select(".comics-detail__desc").text().read();
	let url = format!("{}/comic/{}", BASE_URL, manga_id);
	let categories_array = html.select(".tag-list").select("span").array();
	let status_str = html
		.select("meta[name='og:novel:status']")
		.attr("content")
		.read();

	let mut categories: Vec<String> = Vec::new();
	for category in categories_array {
		let category_str = category.as_node()?.text().read();
		if category_str != "連載中" || category_str != "已完結" || category_str.is_empty() {
			categories.push(category_str);
		}
	}

	let status = if status_str.contains("連載中") {
		MangaStatus::Ongoing
	} else if status_str.contains("已完結") {
		MangaStatus::Completed
	} else {
		MangaStatus::Unknown
	};

	let manga = Manga {
		id: manga_id,
		cover,
		title,
		author: author.clone(),
		artist: author,
		description,
		url,
		categories,
		status,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Scroll,
	};

	Ok(manga)
}

pub fn get_chapter_list(html: Node, manga_id: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	let mut new = true;
	let mut insert = 0;
	for item in html.select(".comics-chapters").array() {
		let chapter_item = item.as_node()?;

		let id: String = String::from(
			chapter_item
				.select("a")
				.attr("href")
				.read()
				.split('=')
				.last()
				.unwrap(),
		);
		let title = chapter_item.select("span").text().read();
		let index: f32 = id.parse().unwrap();
		let url = format!("{}/comic/chapter/{}/0_{}.html", BASE_URL, manga_id, id);

		let chapter = Chapter {
			id: id.clone(),
			title,
			volume: -1.0,
			chapter: index + 1.0,
			date_updated: -1.0,
			scanlator: String::new(),
			url,
			lang: String::from("zh"),
		};

		if id == String::from("0") {
			new = false;
		}
		chapters.insert(insert, chapter);
		if !new {
			insert += 1;
		}
	}

	Ok(chapters)
}

pub fn get_page_list(html: Node) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	let mut index = 0;
	for item in html.select("amp-img[data-v-25d25a4e]").array() {
		let url = item.as_node()?.attr("src").read();

		let page = Page {
			index,
			url,
			base64: String::new(),
			text: String::new(),
		};
		pages.push(page);
		index += 1;
	}

	Ok(pages)
}
