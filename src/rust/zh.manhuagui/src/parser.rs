use crate::{
	decoder::{decompress_from_base64, Decoder},
	helper::{self, encode_uri},
};

use aidoku::{
	error::Result,
	prelude::*,
	std::html::Node,
	std::Vec,
	std::{net::HttpMethod, net::Request, String},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};
use alloc::{string::ToString, vec};

const BASE_URL: &str = "https://www.manhuagui.com";

const FILTER_REGION: [&str; 7] = [
	"all", "japan", "hongkong", "other", "europe", "china", "korea",
];
const FILTER_GENRE: [&str; 39] = [
	"all",
	"rexue",
	"maoxian",
	"mohuan",
	"shengui",
	"gaoxiao",
	"mengxi",
	"aiqing",
	"kehuan",
	"mofa",
	"gedou",
	"wuxia",
	"jizhan",
	"zhanzheng",
	"jingji",
	"tiyu",
	"xiaoyuan",
	"shenghuo",
	"lizhi",
	"lishi",
	"weiniang",
	"zhainan",
	"funv",
	"danmei",
	"baihe",
	"hougong",
	"zhiyu",
	"meishi",
	"tuili",
	"xuanyi",
	"kongbu",
	"sige",
	"zhichang",
	"zhentan",
	"shehui",
	"yinyue",
	"wudao",
	"zazhi",
	"heidao",
];
const FILTER_AUDIENCE: [&str; 6] = [
	"all", "shaonv", "shaonian", "qingnian", "ertong", "tongyong",
];
const FILTER_PROGRESS: [&str; 3] = ["all", "lianzai", "wanjie"];
const SORT: [&str; 4] = ["index", "update", "view", "rate"];

pub fn parse_home_page(html: Node) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	let ul = "#contList > li";

	for element in html.select(ul).array() {
		let elem = match element.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let manga_id = elem
			.select("a")
			.attr("href")
			.read()
			.replace("/comic/", "")
			.replace('/', "");
		let title = elem.select("a").attr("title").read();
		let manga = Manga {
			id: manga_id.clone(),
			cover: format!("https://cf.hamreus.com/cpic/b/{}.jpg", manga_id),
			title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: format!("{}/comic/{}", BASE_URL, manga_id), //`${this.baseUrl}/comic/${mangaId}`;,
			categories: vec![],
			status: MangaStatus::Completed,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll,
		};
		mangas.push(manga);
	}

	let mut has_next: bool = false;
	for page in html.select("#AspNetPager1 > a").array() {
		let elem_node = match page.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		if elem_node.text().read() == "尾页" {
			has_next = true;
			break;
		}
	}

	html.close();

	Ok(MangaPageResult {
		manga: mangas,
		has_more: has_next,
	})
}

pub fn parse_search_page(html: Node) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	let ul = ".cf > .book-cover > a";

	for element in html.select(ul).array() {
		let elem = match element.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let manga_id = elem
			.attr("href")
			.read()
			.replace("/comic/", "")
			.replace('/', "");
		let title = elem.attr("title").read();
		let manga = Manga {
			id: manga_id.clone(),
			cover: format!("https://cf.hamreus.com/cpic/b/{}.jpg", manga_id),
			title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: format!("{}/comic/{}", BASE_URL, manga_id), //`${this.baseUrl}/comic/${mangaId}`;,
			categories: vec![],
			status: MangaStatus::Completed,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll,
		};
		mangas.push(manga);
	}

	let mut has_next: bool = false;
	for page in html.select("#AspNetPagerResult > a").array() {
		let page_node = match page.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		if page_node.text().read() == "尾页" {
			has_next = true;
			break;
		}
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more: has_next,
	})
}

pub fn parse_manga_details(html: Node, manga_id: String) -> Result<Manga> {
	let title = html.select(".book-title > h1").text().read();
	let author = html
      .select(
        "body > div.w998.bc.cf > div.fl.w728 > div.book-cont.cf > div.book-detail.pr.fr > ul > li:nth-child(2) > span:nth-child(2) > a:nth-child(2)"
      )
      .text()
	  .read();
	let desc = html.select("#intro-cut").text().read();
	let image = format!("https://cf.hamreus.com/cpic/b/{}.jpg", manga_id);
	let url = format!("https://www.manhuagui.com/comic/{}/", manga_id);

	let manga = Manga {
		id: manga_id,
		cover: image,
		title,
		author: author.clone(),
		artist: author,
		description: desc,
		url,
		categories: vec![],
		status: MangaStatus::Ongoing,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Scroll,
	};

	Ok(manga)
}

pub fn get_chapter_list(html: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	let mut index = 1.0;

	let mut div = html.clone();
	let hidden = html.html().read().contains("__VIEWSTATE");
	if hidden {
		let compressed = html.select("#__VIEWSTATE").attr("value").read();
		let decompressed =
			String::from_utf16(&decompress_from_base64(compressed.as_str()).unwrap_or_default())
				.unwrap_or_default();
		div = Node::new_fragment(decompressed.as_bytes()).unwrap_or(div);
	}

	for element in div.select(".chapter-list").array().rev() {
		let chapt_list_div = match element.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};

		for ul_ref in chapt_list_div.select("ul").array() {
			let ul = match ul_ref.as_node() {
				Ok(node) => node,
				Err(_) => continue,
			};

			for li_ref in ul.select("li").array().rev() {
				let elem = match li_ref.as_node() {
					Ok(node) => node,
					Err(_) => continue,
				};

				let url = elem.select("a").attr("href").read();
				let id = url.clone().replace("/comic/", "").replace(".html", "");
				let chapter_id = match id.split('/').last() {
					Some(id) => String::from(id),
					None => String::new(),
				};
				let title = elem.select("a").attr("title").read();
				let chapter_or_volume = title
					.clone()
					.replace(['第', '话', '話', '回', '卷'], " ")
					.parse::<f32>()
					.unwrap_or(index);
				let ch = if title.contains('卷') {
					-1.0
				} else {
					chapter_or_volume
				};
				let vo = if title.contains('卷') {
					chapter_or_volume
				} else {
					-1.0
				};

				let chapter = Chapter {
					id: chapter_id,
					title,
					volume: vo,
					chapter: ch,
					date_updated: -1.0,
					scanlator: String::new(),
					url,
					lang: String::from("zh"),
				};

				chapters.push(chapter);
				index += 1.0;
			}
		}
	}
	chapters.reverse();

	Ok(chapters)
}

pub fn get_page_list(base_url: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	let html = Request::new(base_url.as_str(), HttpMethod::Get).html()?;

	let decoder = Decoder::new(html.html().read());
	let (path, pages_str) = decoder.decode();

	for (index, str) in pages_str.into_iter().enumerate() {
		let encoded_path = helper::encode_uri(&path);
		let url = format!("https://i.hamreus.com{}{}", encoded_path, str);
		pages.push(Page {
			index: index as i32,
			url,
			base64: String::new(),
			text: String::new(),
		});
	}

	Ok(pages)
}

// FILTER

pub fn get_filtered_url(filters: Vec<Filter>, page: i32, url: &mut String) {
	let mut is_searching = false;
	let mut search_string = String::new();
	url.push_str(BASE_URL);

	let mut region: &str = "all";
	let mut genre: &str = "all";
	let mut audience: &str = "all";
	let mut progress: &str = "all";
	let mut sort_by = SORT[0];

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					search_string
						.push_str(encode_uri(&filter_value.read().to_lowercase()).as_str());
					is_searching = true;
				}
			}
			FilterType::Select => {
				let index = filter.value.as_int().unwrap_or(0) as usize;
				match filter.name.as_str() {
					"地区" => region = FILTER_REGION[index],
					"剧情" => genre = FILTER_GENRE[index],
					"受众" => audience = FILTER_AUDIENCE[index],
					"进度" => progress = FILTER_PROGRESS[index],
					_ => continue,
				};
			}
			FilterType::Sort => {
				let Ok(obj) = filter.value.as_object() else {
					continue;
				};
				let index = obj.get("index").as_int().unwrap_or(0) as usize;
				sort_by = SORT[index];
			}
			_ => continue,
		}
	}

	if is_searching {
		url.push_str("/s");
		let search_page_str = format!("/{}_p{}.html", search_string, page.to_string());
		url.push_str(search_page_str.as_str());
	} else {
		url.push_str("/list");

		let mut filter_values: Vec<&str> = Vec::new();
		for val in [region, genre, audience, progress] {
			if val != "all" {
				filter_values.push(val);
			}
		}

		let mut filter_str = filter_values.join("_");

		if !filter_str.is_empty() {
			filter_str = format!("/{}", filter_str)
		}

		let page_str = format!("/{}_p{}.html", sort_by, page.to_string());

		url.push_str(filter_str.as_str());
		url.push_str(page_str.as_str())
	}
}
