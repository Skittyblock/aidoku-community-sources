#![no_std]
mod helper;
use aidoku::{
	error::{AidokuError, Result},
	prelude::*,
	std::{
		html::Node,
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};
use helper::{
	append_protocol, category_parser, extract_f32_from_string, text_with_newlines, urlencode,
};

static mut CACHED_MANGA_ID: Option<String> = None;
static mut CACHED_MANGA: Option<Vec<u8>> = None;

fn cache_manga_page(id: &str) {
	unsafe {
		if CACHED_MANGA.is_some() && id == CACHED_MANGA_ID.clone().unwrap().as_str() {
			return;
		}

		CACHED_MANGA_ID = Some(String::from(id));
		CACHED_MANGA = Some(
			Request::new(
				format!("http://truyentranh86.com{id}").as_str(),
				HttpMethod::Get,
			)
			.data(),
		);
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut url = format!("http://truyentranh86.com/search.php?act=timnangcao&page={page}");
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
						1 => "and",
						2 => "or",
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
	let html = Request::new(&url, HttpMethod::Get).html()?;
	let node = html.select("div#tblChap figure.col");
	let elems = node.array();
	let mut manga = Vec::with_capacity(elems.len());
	let has_more = html
		.select("div#tblChap p.page a:contains(Cuối)")
		.array()
		.len() > 0;
	for elem in elems {
		let manga_node = elem.as_node().expect("node array");
		let title = manga_node.select("figcaption h3 a").text().read();
		let url = manga_node.select("figcaption h3 a").attr("href").read();
		let id = url.replace("http://truyentranh86.com", "");
		let cover = append_protocol(manga_node.select("img").attr("src").read());
		manga.push(Manga {
			id,
			cover,
			title: String::from(title.replace("[TT8]", "").trim()),
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url,
			categories: Vec::new(),
			status: MangaStatus::Unknown,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Rtl,
		})
	}
	Ok(MangaPageResult { manga, has_more })
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	cache_manga_page(&id);
	let html = Node::new(unsafe { &CACHED_MANGA.clone().unwrap() })?;
	let cover = append_protocol(html.select("img.thumbnail").attr("src").read());
	let title = String::from(
		html.select("h1.fs-5")
			.text()
			.read()
			.replace("Truyện Tranh", "")
			.trim(),
	);
	let author = html
		.select("span[itemprop=author]")
		.array()
		.filter_map(|str| {
			let string = str.as_node().expect("node array").text().read();
			if string.is_empty() {
				None
			} else {
				Some(string)
			}
		})
		.collect::<Vec<_>>()
		.join(", ");
	let description =
		text_with_newlines(html.select("div.card-body.border-start.border-info.border-3"));
	let url = format!("http://truyentranh86.com{}", id);
	let categories = html
		.select("a[itemprop=genre]")
		.array()
		.filter_map(|str| {
			let string = str.as_node().expect("node array").text().read();
			if string.is_empty() {
				None
			} else {
				Some(string)
			}
		})
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
		.select("a[itemprop=contentRating][href='http://truyentranh86.com/DoTuoi/18/']")
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
	let html = Node::new(unsafe { &CACHED_MANGA.clone().unwrap() })?;
	let node = html.select("ul#ChapList li");
	let mut scanlator = html
		.select("b[itemprop=editor]")
		.array()
		.filter_map(|str| {
			let string = str.as_node().expect("node array").text().read();
			if string.is_empty() {
				None
			} else {
				Some(string)
			}
		})
		.collect::<Vec<_>>()
		.join(", ");
	if &scanlator == "Unknown" {
		scanlator = String::new();
	}
	let elems = node.array();
	let mut chapters = Vec::with_capacity(elems.len());
	for elem in elems {
		let chapter_node = elem.as_node().expect("node array");
		let url = chapter_node.select("a").attr("href").read();
		let chapter_id = url.replace("http://truyentranh86.com", "");
		let title = chapter_node
			.select("small.gray-500")
			.text()
			.read()
			.replace("- ", "");
		let vol_chap = chapter_node.select("strong").text().read();
		let numbers = extract_f32_from_string(String::new(), vol_chap.clone());
		let (volume, chapter) =
			if numbers.len() > 1 && vol_chap.to_ascii_lowercase().contains("vol") {
				(numbers[0], numbers[1])
			} else if !numbers.is_empty() {
				(-1.0, numbers[0])
			} else {
				(-1.0, -1.0)
			};
		let date_updated = chapter_node
			.select("time")
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
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("http://truyentranh86.com{chapter_id}");
	let html = Request::new(&url, HttpMethod::Get).html()?;
	let node = html.select("div.page-chapter");
	let elems = node.array();
	let mut pages = Vec::with_capacity(elems.len());
	for (idx, elem) in elems.enumerate() {
		let page_node = elem.as_node().expect("node array");
		pages.push(Page {
			index: idx as i32,
			url: page_node.select("img").attr("src").read(),
			..Default::default()
		})
	}
	Ok(pages)
}

#[modify_image_request]
fn modify_image_request(_request: Request) {}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	// http://truyentranh86.com/bon-te-tu-chinh-la-tien-dao-29234/828715-chap-247
	// ['http:', '', 'truyentranh86.com', 'bon-te-tu-chinh-la-tien-dao-29234',
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
