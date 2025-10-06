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

fn extract_chapter_number(title: &str) -> Option<f32> {
	let keywords = ["话", "話", "章", "回", "卷"];
	for &kw in &keywords {
		if let Some(pos) = title.rfind(kw) {
			let before = &title[..pos];
			let mut start = pos;
			while start > 0 && (before.as_bytes()[start - 1].is_ascii_digit() || before.as_bytes()[start - 1] == b'.') {
				start -= 1;
			}
			if start < pos {
				let num_str = &before[start..];
				if let Ok(num) = num_str.parse::<f32>() {
					return Some(num);
				}
			}
		}
	}
	None
}

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
			url: format!("{}/comic/{}", crate::get_base_url(), manga_id), //`${this.baseUrl}/comic/${mangaId}`;,
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
		let text = elem_node.text().read();
		if text == "尾页" || text == "尾頁" {
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
			url: format!("{}/comic/{}", crate::get_base_url(), manga_id), //`${this.baseUrl}/comic/${mangaId}`;,
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
		let text = page_node.text().read();
		if text == "尾页" || text == "尾頁" {
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
	let mut authors = Vec::new();
	for author_link in html.select("ul.detail-list li:nth-child(2) span:nth-child(2) a").array() {
		if let Ok(author_node) = author_link.as_node() {
			let author_text = author_node.text().read();
			if !author_text.is_empty() {
				authors.push(author_text);
			}
		}
	}
	let author = authors.join(", ");
	let desc = html.select("#intro-cut").text().read();
	let image = format!("https://cf.hamreus.com/cpic/b/{}.jpg", manga_id);
	let url = format!("{}/comic/{}/", crate::get_base_url(), manga_id);

	let status_text = html.select("li.status").text().read();
	let status = if status_text.contains("已完结") || status_text.contains("已完結") {
		MangaStatus::Completed
	} else {
		MangaStatus::Ongoing
	};

	let mut categories = Vec::new();
	for category in html.select("ul.detail-list li:nth-child(2) span:nth-child(1) a").array() {
		if let Ok(cat_node) = category.as_node() {
			let cat_text = cat_node.text().read();
			if !cat_text.is_empty() {
				categories.push(cat_text);
			}
		}
	}

	let manga = Manga {
		id: manga_id,
		cover: image,
		title,
		author: author.clone(),
		artist: author,
		description: desc,
		url,
		categories,
		status,
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

	// Parse scanlators from h4 tags
	let mut scanlators: Vec<String> = Vec::new();
	for h4 in div.select("h4").array() {
		if let Ok(h4_node) = h4.as_node() {
			let scanlator = h4_node.select("span").text().read();
			scanlators.push(scanlator);
		}
	}
	scanlators.reverse(); // Reverse to match the .rev() order of chapter-list

	let mut scanlator_index = 0;

	for element in div.select(".chapter-list").array().rev() {
		let chapt_list_div = match element.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};

		let scanlator = if scanlator_index < scanlators.len() {
			scanlators[scanlator_index].clone()
		} else {
			String::new()
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
				let chapter_id = match id.split('/').next_back() {
					Some(id) => String::from(id),
					None => String::new(),
				};
				let mut title = elem.select("a").attr("title").read();
				let chapter_or_volume = extract_chapter_number(&title).unwrap_or(index);
				let (ch, vo) = if title.trim().ends_with('卷') {
					(-1.0, chapter_or_volume)
				} else {
					(chapter_or_volume, -1.0)
				};

				// Add page count if available
				let page_text = elem.select("i").text().read();
				if !page_text.is_empty() {
					title = format!("{} ({})", title, page_text);
				}

				let chapter = Chapter {
					id: chapter_id,
					title,
					volume: vo,
					chapter: ch,
					date_updated: -1.0,
					scanlator: scanlator.clone(),
					url,
					lang: String::from("zh"),
				};

				chapters.push(chapter);
				index += 1.0;
			}
		}
		scanlator_index += 1;
	}
	chapters.reverse();

	Ok(chapters)
}

pub fn get_page_list(base_url: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	let request = Request::new(base_url.as_str(), HttpMethod::Get)
		.header("Referer", crate::get_base_url())
		.header("User-Agent", crate::USER_AGENT)
		.header("Accept-Language", "zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7")
		.header("Cookie", "device_view=pc");
	let html = request.html()?;

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
	url.push_str(crate::get_base_url());

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
