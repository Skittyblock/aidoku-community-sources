#![no_std]
mod helper;
use aidoku::{
	error::{AidokuError, Result},
	prelude::*,
	std::{
		defaults::defaults_get,
		html::Node,
		net::{HttpMethod, Request},
		String, StringRef, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};
use helper::{
	category_parser, extract_f32_from_string, text_with_newlines, url_from_css_style, urlencode,
};

static mut CACHED_MANGA_ID: Option<String> = None;
static mut CACHED_MANGA: Option<Vec<u8>> = None;

fn get_base_url() -> String {
	defaults_get("sourceURL")
		.as_string()
		.unwrap_or_else(|_| StringRef::from("https://mangaxy.com"))
		.read()
}

fn cache_manga_page(id: &str) {
	let base_url = get_base_url();

	unsafe {
		if CACHED_MANGA.is_some() && id == CACHED_MANGA_ID.clone().unwrap().as_str() {
			return;
		}

		CACHED_MANGA_ID = Some(String::from(id));
		CACHED_MANGA =
			Some(Request::new(format!("{base_url}{id}").as_str(), HttpMethod::Get).data());
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let base_url = get_base_url();
	let mut url = format!("{base_url}/search.php?act=search&page={page}");
	let mut included_tags = Vec::new();
	let mut excluded_tags = Vec::new();
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				url.push_str("&q=");
				url.push_str(&urlencode(filter.value.as_string()?.read()));
			}
			FilterType::Author => {
				url.push_str("&TacGia=");
				url.push_str(&urlencode(filter.value.as_string()?.read()));
			}
			FilterType::Genre => {
				if let Ok(id) = filter.object.get("id").as_string() {
					let id = id.read();
					match filter.value.as_int().unwrap_or(-1) {
						0 => excluded_tags.push(id),
						1 => included_tags.push(id),
						_ => continue,
					}
				}
			}
			FilterType::Select => match filter.name.as_str() {
				"Kiểu tìm thể loại" => {
					url.push_str("&andor=");
					url.push_str(match filter.value.as_int().unwrap_or(-1) {
						0 => "and",
						1 => "or",
						_ => "",
					})
				}
				"Dành cho" => {
					url.push_str("&danhcho=");
					url.push_str(match filter.value.as_int().unwrap_or(-1) {
						1 => "gai",
						2 => "trai",
						3 => "nit",
						_ => "",
					});
				}
				"Độ tuổi" => {
					url.push_str("&DoTuoi=");
					url.push_str(match filter.value.as_int().unwrap_or(-1) {
						1 => "13",
						2 => "14",
						3 => "15",
						4 => "16",
						5 => "17",
						6 => "18",
						_ => "",
					});
				}
				"Tình trạng" => {
					url.push_str("&TinhTrang=");
					url.push_str(match filter.value.as_int().unwrap_or(-1) {
						1 => "Ongoing",
						2 => "Complete",
						3 => "Drop",
						_ => "",
					});
				}
				"Quốc gia" => {
					url.push_str("&quocgia=");
					url.push_str(match filter.value.as_int().unwrap_or(-1) {
						1 => "nhat",
						2 => "trung",
						3 => "han",
						4 => "vietnam",
						_ => "",
					});
				}
				"Kiểu đọc" => {
					url.push_str("&KieuDoc=");
					url.push_str(
						match filter.value.as_int().unwrap_or(-1) {
							1 => urlencode(String::from("chưa xác định")),
							2 => urlencode(String::from("xem từ phải qua trái")),
							3 => urlencode(String::from("xem từ trái qua phải")),
							_ => String::new(),
						}
						.as_str(),
					);
				}
				"Sắp xếp theo" => {
					url.push_str("&sort=");
					url.push_str(match filter.value.as_int().unwrap_or(-1) {
						0 => "chap",
						1 => "truyen",
						2 => "xem",
						3 => "ten",
						_ => "",
					});
				}
				_ => continue,
			},
			_ => continue,
		}
	}
	url.push_str(format!("&baogom={}", included_tags.join(",")).as_str());
	url.push_str(format!("&khonggom={}", excluded_tags.join(",")).as_str());
	let html = Request::new(&url, HttpMethod::Get).html();
	let node = html.select("div#tblChap div.thumb");
	let elems = node.array();
	let mut manga = Vec::with_capacity(elems.len());
	let has_more = html
		.select("div#tblChap p.page a:contains(Cuối)")
		.array()
		.len() > 0;
	for elem in elems {
		let manga_node = elem.as_node();
		let title = manga_node.select("a.name").text().read();
		let url = manga_node.select("a.name").attr("href").read();
		let id = url.replace(&base_url, "");
		let cover = url_from_css_style(manga_node.select("img").attr("style").read());
		manga.push(Manga {
			id,
			cover,
			title: String::from(title.replace("MỚI", "").replace("18+", "").trim()),
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url,
			categories: Vec::new(),
			status: MangaStatus::Unknown,
			nsfw: if title.clone().contains("18+") {
				MangaContentRating::Nsfw
			} else {
				MangaContentRating::Safe
			},
			viewer: MangaViewer::Rtl,
		})
	}
	Ok(MangaPageResult { manga, has_more })
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	cache_manga_page(&id);
	let html = Node::new(unsafe { &CACHED_MANGA.clone().unwrap() });
	let cover = url_from_css_style(html.select("div.detail-top-right img").attr("style").read());
	let title = String::from(html.select("h1.comics-title").text().read().trim());
	let author = html.select("div.created-by").text().read();
	let description = text_with_newlines(html.select("div.manga-info p"));
	let url = format!("{}{}", get_base_url(), id);
	let categories = html
		.select("div.top-comics-type")
		.text()
		.read()
		.split(" / ")
		.map(String::from)
		.collect::<Vec<_>>();
	let status = match html
		.select("div.manga-info a[href*=TinhTrang]")
		.text()
		.read()
		.trim()
	{
		"Đang tiến hành" => MangaStatus::Ongoing,
		"Đã hoàn thành" => MangaStatus::Completed,
		"Tạm ngưng" => MangaStatus::Hiatus,
		_ => MangaStatus::Unknown,
	};
	let (mut nsfw, viewer) = category_parser(&categories);
	nsfw = if html
		.select("a[itemprop=contentRating][href='https://mangaxy.com/DoTuoi/18/']")
		.array()
		.len() > 0
	{
		MangaContentRating::Nsfw
	} else {
		nsfw
	};
	Ok(Manga {
		id,
		cover,
		title,
		author,
		artist: String::new(),
		description,
		url,
		categories,
		status,
		nsfw,
		viewer,
	})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	cache_manga_page(&id);
	let html = Node::new(unsafe { &CACHED_MANGA.clone().unwrap() });
	let node = html.select("a.episode-item");
	let mut scanlator = html.select("div.manga-info a[href*=Nguon]").text().read();
	if &scanlator == "Unknown" {
		scanlator = String::new();
	}
	let elems = node.array();
	let mut chapters = Vec::with_capacity(elems.len());
	for elem in elems {
		let chapter_node = elem.as_node();
		let url = chapter_node.attr("href").read();
		let chapter_id = url.replace(&get_base_url(), "");
		let full_title = chapter_node.select("div.episode-title").text().read();
		let title = chapter_node
			.select("div.episode-title div.chap-name")
			.text()
			.read();
		let vol_chap = full_title.replace(&title, "");
		let numbers = extract_f32_from_string(String::new(), String::from(&vol_chap));
		let (volume, chapter) =
			if numbers.len() > 1 && vol_chap.to_ascii_lowercase().contains("vol") {
				(numbers[0], numbers[1])
			} else if !numbers.is_empty() {
				(-1.0, numbers[0])
			} else {
				(-1.0, -1.0)
			};
		let date_updated = chapter_node
			.select("div.episode-date time")
			.attr("datetime")
			.0
			.as_date("yyyy-MM-dd HH:mm:ss", None, Some("Asia/Ho_Chi_Minh"))
			.unwrap_or(-1.0);
		chapters.push(Chapter {
			id: chapter_id,
			title: String::from(title.trim()),
			volume,
			chapter,
			date_updated,
			scanlator: String::from(&scanlator),
			url,
			lang: String::from("vi"),
		});
	}
	Ok(chapters)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	let url = format!("{}{id}", get_base_url());
	let html = Request::new(&url, HttpMethod::Get).html();
	let node = html.select("div.page-chapter");
	let elems = node.array();
	let mut pages = Vec::with_capacity(elems.len());
	for (idx, elem) in elems.enumerate() {
		let page_node = elem.as_node();
		pages.push(Page {
			index: idx as i32,
			url: page_node.select("img").attr("src").read(),
			base64: String::new(),
			text: String::new(),
		})
	}
	Ok(pages)
}

#[modify_image_request]
fn modify_image_request(_request: Request) {}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	// https://mangaxy.com/bon-te-tu-chinh-la-tien-dao-29234/828715-chap-247
	// ['https:', '', 'mangaxy.com', 'bon-te-tu-chinh-la-tien-dao-29234',
	// '828715-chap-247']
	let split_url = url.split('/').map(String::from).collect::<Vec<_>>();
	if split_url.len() > 4 {
		Ok(DeepLink {
			manga: Some(get_manga_details(format!("/{}/", split_url[3]))?),
			chapter: if split_url[4].is_empty() {
				None
			} else {
				Some(Chapter {
					id: format!("/{}/{}", split_url[3], split_url[4]),
					title: String::new(),
					volume: -1.0,
					chapter: -1.0,
					date_updated: -1.0,
					scanlator: String::new(),
					url,
					lang: String::from("vi"),
				})
			},
		})
	} else {
		Err(AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		})
	}
}
