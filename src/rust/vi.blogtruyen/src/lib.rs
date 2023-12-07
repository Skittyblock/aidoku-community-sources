#![no_std]
mod helper;
use crate::helper::{
	category_parser, extract_f32_from_string, genre_map, status_from_string, text_with_newlines,
	urlencode,
};
use aidoku::{
	error::Result,
	prelude::*,
	std::{
		html::Node,
		json::parse,
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, Page,
};

static mut CACHED_MANGA_ID: Option<String> = None;
static mut CACHED_MANGA: Option<Vec<u8>> = None;
static BASE_URL: &str = "https://blogtruyenmoi.com";

fn cache_manga_page(id: &str) {
	if unsafe { CACHED_MANGA_ID.is_some() } && unsafe { CACHED_MANGA_ID.clone().unwrap() } == id {
		return;
	}

	unsafe {
		CACHED_MANGA =
			Some(Request::new(format!("{BASE_URL}{id}").as_str(), HttpMethod::Get).data());
		CACHED_MANGA_ID = Some(String::from(id));
	};
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut included_tags: Vec<String> = Vec::new();
	let mut excluded_tags: Vec<String> = Vec::new();
	let mut title: String = String::new();
	let mut author: String = String::new();
	let mut status: i32 = -1;
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				title = urlencode(filter.value.as_string()?.read());
			}
			FilterType::Author => {
				author = urlencode(filter.value.as_string()?.read());
			}
			FilterType::Genre => {
				let id = if let Ok(genre_id) = filter.object.get("id").as_string() {
					genre_id.read()
				} else {
					genre_map(filter.name)
				};
				match filter.value.as_int().unwrap_or(-1) {
					0 => excluded_tags.push(id),
					1 => included_tags.push(id),
					_ => continue,
				}
			}
			_ => match filter.name.as_str() {
				"Trạng thái" => {
					status = filter.value.as_int().unwrap_or(-1) as i32;
				}
				_ => continue,
			},
		}
	}

	let mut manga_arr: Vec<Manga> = Vec::new();
	if !included_tags.is_empty()
		|| !excluded_tags.is_empty()
		|| !title.is_empty()
		|| !author.is_empty()
		|| status != 0
	{
		let included = if !included_tags.is_empty() {
			included_tags.join(",")
		} else {
			String::from("-1")
		};
		let excluded = if !excluded_tags.is_empty() {
			excluded_tags.join(",")
		} else {
			String::from("-1")
		};
		let html = Request::new(
			format!(
				// This page has a scanlator search feature, maybe add that when Aidoku has it
				"{BASE_URL}/timkiem/nangcao/1/{status}/{included}/{excluded}?txt={title}&aut={author}&p={page}&gr="
			)
			.as_str(),
			HttpMethod::Get,
		)
		.html()?;
		for (url_info, info) in html.select("div.list > p > span.tiptip > a").array().zip(
			html.select("div.list > div.tiptip-content > div.row")
				.array(),
		) {
			let url_node = url_info.as_node().expect("node array");
			let info_node = info.as_node().expect("node array");
			let title = info_node.select("div.col-sm-8 > div.al-c").text().read();
			let description = text_with_newlines(info_node.select("div.col-sm-8 > div.al-j"));
			let cover = info_node.select("div.col-sm-4 > img").attr("src").read();
			let id = url_node.attr("href").read();
			let url = format!("{BASE_URL}{id}");
			manga_arr.push(Manga {
				id,
				cover,
				title: String::from(title.trim()),
				description: String::from(description.trim()),
				url,
				..Default::default()
			});
		}
		Ok(MangaPageResult {
			manga: manga_arr,
			has_more: html.select("a[title=\"Trang cuối\"]").array().len() > 0,
		})
	} else {
		let html =
			Request::new(format!("{BASE_URL}/page-{page}").as_str(), HttpMethod::Get).html()?;

		// Get daily featured mangas
		if page == 1 {
			for (image, tooltip) in html
				.select("a.tiptip")
				.array()
				.zip(html.select("div.tiptip-content").array())
			{
				let image_node = image.as_node().expect("node array");
				let tooltip_node = tooltip.as_node().expect("node array");
				let id = image_node.attr("href").read();
				let url = format!("{BASE_URL}{id}");
				let cover = image_node.select("img").attr("src").read();
				let title = tooltip_node.select("p.bold").text().read();
				let description = text_with_newlines(tooltip_node.select("p:not(.bold)"));
				manga_arr.push(Manga {
					id,
					cover,
					title,
					description: String::from(description.trim()),
					url,
					..Default::default()
				});
			}
		}
		for info in html.select("div.storyitem").array() {
			let info_node = info.as_node().expect("node array");
			let title = info_node.select("div.fl-l > a").attr("title").read();
			let description = text_with_newlines(info_node.select("div.fl-r > p.al-j"));
			let cover = info_node
				.select("div.fl-l img:not(.imgBack)")
				.attr("src")
				.read();
			let id = info_node.select("div.fl-l > a").attr("href").read();
			let url = format!("{BASE_URL}{id}");
			let categories = info_node
				.select("div.category > a")
				.array()
				.map(|category| category.as_node().expect("node array").attr("title").read())
				.collect::<Vec<String>>();
			let (nsfw, viewer) = category_parser(&categories);
			manga_arr.push(Manga {
				id,
				cover,
				title: String::from(title.trim()),
				description: String::from(description.trim()),
				url,
				categories,
				nsfw,
				viewer,
				..Default::default()
			});
		}
		Ok(MangaPageResult {
			manga: manga_arr,
			has_more: html.select("a[title=\"Trang cuối\"]").array().len() > 0,
		})
	}
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	cache_manga_page(&id);
	let html = unsafe { Node::new(&CACHED_MANGA.clone().unwrap()) }?;
	let url = format!("{BASE_URL}{id}");
	let title = html
		.select("div.thumbnail > img")
		.attr("alt")
		.read()
		.replace("truyện tranh", "");
	let cover = html.select("div.thumbnail > img").attr("src").read();
	let description =
		text_with_newlines(html.select("section.manga-detail > div.detail > div.content"));
	let author = html
		.select("div.description > p:contains(Tác giả) > a")
		.array()
		.map(|val| val.as_node().expect("node array").text().read())
		.collect::<Vec<String>>()
		.join(", ");
	let categories = html
		.select("span.category > a")
		.array()
		.map(|val| val.as_node().expect("node array").text().read())
		.collect::<Vec<String>>();
	let status = status_from_string(
		html.select("p:contains(Trạng thái) > span.color-red")
			.text()
			.read(),
	);
	let (mut nsfw, viewer) = category_parser(&categories);
	if html
		.select("div.modal-header:contains(Cảnh báo độ tuổi)")
		.array()
		.len() > 0
	{
		nsfw = MangaContentRating::Nsfw;
	}
	Ok(Manga {
		id,
		cover,
		title: String::from(title.trim()),
		author,
		description: String::from(description.trim()),
		url,
		categories,
		status,
		nsfw,
		viewer,
		..Default::default()
	})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	cache_manga_page(&id);
	let html = unsafe { Node::new(&CACHED_MANGA.clone().unwrap()) }?;
	let mut scanlator = html
		.select("span.translater")
		.array()
		.map(|val| val.as_node().expect("node array").text().read())
		.collect::<Vec<String>>();
	scanlator.dedup();
	let scanlator_string = scanlator.join(", ");
	let manga_title = html
		.select("div.thumbnail > img")
		.attr("alt")
		.read()
		.replace("truyện tranh", "");
	let mut chapter_arr: Vec<Chapter> = Vec::new();
	for chapter_item in html.select("p[id^=\"chapter\"]").array() {
		let chapter_node = chapter_item.as_node().expect("node array");
		let chapter_id = chapter_node.select("span.title > a").attr("href").read();
		let mut title = chapter_node
			.select("span.title > a")
			.text()
			.read()
			.replace(manga_title.trim(), "");
		let numbers = extract_f32_from_string(String::from(&manga_title), String::from(&title));
		let (volume, chapter) = if numbers.len() > 1 && title.to_ascii_lowercase().contains("vol") {
			(numbers[0], numbers[1])
		} else if !numbers.is_empty() {
			(-1.0, numbers[0])
		} else {
			(-1.0, -1.0)
		};
		if chapter >= 0.0 {
			let splitter = format!(" {}", chapter);
			if title.contains(&splitter) {
				let split = title.splitn(2, &splitter).collect::<Vec<&str>>();
				title = String::from(split[1]).replacen(|char| char == ':' || char == '-', "", 1);
			}
		}
		let date_updated = chapter_node
			.select("span.publishedDate")
			.text()
			.0
			.as_date("dd/MM/yyyy HH:mm", Some("en_US"), Some("Asia/Ho_Chi_Minh"))
			.unwrap_or(-1.0);
		let url = format!("{BASE_URL}{chapter_id}");
		chapter_arr.push(Chapter {
			id: chapter_id,
			title: String::from(title.trim()),
			volume,
			chapter,
			date_updated,
			scanlator: String::from(&scanlator_string),
			url,
			lang: String::from("vi"),
		});
	}
	Ok(chapter_arr)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{BASE_URL}{chapter_id}");
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	let mut page_arr: Vec<Page> = Vec::new();
	let mut page_index = 0;
	for page_item in html.select("article#content > img").array() {
		let page_node = page_item.as_node().expect("node array");
		page_arr.push(Page {
			index: page_index,
			url: page_node.attr("src").read(),
			base64: String::new(),
			text: String::new(),
		});
		page_index += 1;
	}

	// some chapters push pages from script
	// refer to tachiyomiorg/tachiyomi-extensions#10615
	let script = html
		.select("article#content > script:contains(listImageCaption)")
		.html()
		.read();
	if !script.is_empty() {
		let images_array_string = script.split(';').collect::<Vec<&str>>()[0]
			.split('=')
			.collect::<Vec<&str>>()[1];
		let val = parse(images_array_string.as_bytes())?;
		if let Ok(images_array) = val.as_array() {
			for image in images_array {
				let image_object = image.as_object()?;
				page_arr.push(Page {
					index: page_index,
					url: image_object.get("url").as_string()?.read(),
					..Default::default()
				});
				page_index += 1;
			}
		}
	}
	Ok(page_arr)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.header("Referer", BASE_URL);
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	// https://blogtruyenmoi.com/19588/uchi-no-hentai-maid-ni-osowareteru
	// 'https:', '', 'blogtruyenmoi.com', '19588', 'uchi-no-hentai-maid-ni-osowareteru'
	// https://blogtruyenmoi.com/c694877/shounen-no-abyss-chap-93-ket-thuc
	// 'https:', '', 'blogtruyenmoi.com', 'c694877', 'shounen-no-abyss-chap-93-ket-thuc'
	let split = url.split('/').collect::<Vec<&str>>();
	let id = format!("/{}", &split[3..].join("/"));
	if split[3].contains('c') {
		let html = Request::new(format!("{BASE_URL}{id}").as_str(), HttpMethod::Get).html()?;
		let manga_id = html
			.select("div.breadcrumbs > a:nth-child(2)")
			.attr("href")
			.read();
		let manga = get_manga_details(manga_id)?;
		let mut title = html.select("header h1").text().read();
		let numbers = extract_f32_from_string(String::from(&manga.title), String::from(&title));
		let (volume, chapter) = if numbers.len() > 1 && title.to_ascii_lowercase().contains("vol") {
			(numbers[0], numbers[1])
		} else if !numbers.is_empty() {
			(-1.0, numbers[0])
		} else {
			(-1.0, -1.0)
		};
		if chapter >= 0.0 {
			let splitter = format!(" {}", chapter);
			if title.contains(&splitter) {
				let split = title.splitn(2, &splitter).collect::<Vec<&str>>();
				title = String::from(split[1]).replacen(|char| char == ':' || char == '-', "", 1);
			}
		}
		let chapter = Chapter {
			id,
			title,
			volume,
			chapter,
			date_updated: -1.0,
			scanlator: String::new(),
			url,
			lang: String::from("vi"),
		};
		Ok(DeepLink {
			manga: Some(manga),
			chapter: Some(chapter),
		})
	} else {
		let manga = get_manga_details(id)?;
		Ok(DeepLink {
			manga: Some(manga),
			chapter: None,
		})
	}
}
