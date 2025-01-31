use aidoku::{
	error::{AidokuError, Result},
	prelude::*,
	std::{
		html::Node,
		json,
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, MangaStatus, Page,
};

use crate::helper::*;

pub static mut CACHED_MANGA_URL: Option<String> = None;
pub static mut CACHED_MANGA: Option<Vec<u8>> = None;

pub fn cache_manga_page(url: &str) {
	unsafe {
		if CACHED_MANGA.is_some() && CACHED_MANGA_URL.clone().unwrap() == url {
			return;
		}

		CACHED_MANGA_URL = Some(String::from(url));
		CACHED_MANGA = Some(Request::new(url, HttpMethod::Get).data());
	}
}

pub struct MangaChanSource {
	pub base_url: &'static str,
	pub vol_chap_parser: fn(String, String) -> (f32, f32),
	pub author_selector: &'static str,
	pub custom_new_path: Option<&'static str>,
}

impl MangaChanSource {
	fn parse_manga_list(&self, html: Node, page_size: usize) -> Result<MangaPageResult> {
		let node = html.select("div.content_row");
		let elems = node.array();
		let has_more = elems.len() == page_size;
		let manga = elems
			.map(|elem| {
				let manga_node = elem.as_node().expect("node array");
				let title = manga_node.select("div.manga_row1 h2 a").text().read();
				let url = manga_node.select("div.manga_row1 h2 a").attr("href").read();
				let id = strip_base_url(&url).into();
				let cover = manga_node
					.select("div.manga_images img")
					.attr("src")
					.read()
					.replace("_blur/", "/");
				let author = manga_node.select("div.manga_row2 h3.item2").text().read();
				let description = text_with_newlines(manga_node.select("div.tags"));
				let mut categories = manga_node
					.select("div.manga_row3:contains(Тэги) a")
					.array()
					.map(|elem| elem.as_node().expect("node array").text().read())
					.collect::<Vec<_>>();
				categories.push(
					manga_node
						.select("div.manga_row1 a[href*=/type/]")
						.text()
						.read(),
				);
				let (nsfw, viewer) = category_parser(&categories);
				let status_str = manga_node
					.select("div.manga_row3:contains(Статус (томов)) div.item2")
					.text()
					.read();
				let status = if status_str.contains("перевод продолжается") {
					MangaStatus::Ongoing
				} else if status_str.contains("перевод завершен") {
					MangaStatus::Completed
				} else {
					MangaStatus::Unknown
				};
				Manga {
					id,
					cover,
					title,
					author: String::from(author.trim()),
					artist: String::new(),
					description,
					url,
					categories,
					status,
					nsfw,
					viewer,
				}
			})
			.collect::<Vec<_>>();
		Ok(MangaPageResult { manga, has_more })
	}

	pub fn get_manga_list(&self, filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
		let mut title = String::new();
		let mut sort = String::from("&n=");
		let mut order_by_date_when_search = false;
		let mut tags = Vec::new();
		for filter in filters {
			match filter.kind {
				FilterType::Title => title = urlencode(filter.value.as_string()?.read()),
				FilterType::Sort => {
					if let Ok(value) = filter.value.as_object() {
						let index = value.get("index").as_int().unwrap_or(0);
						let asc = value.get("ascending").as_bool().unwrap_or(false);
						sort.push_str(match index {
							0 => "date",
							1 => "fav",
							2 => "abc",
							3 => "ch",
							_ => "",
						});
						sort.push_str(match asc {
							true => "asc",
							false => "desc",
						});
						if &sort == "&n=datedesc" {
							sort = String::new();
						}
					}
				}
				FilterType::Genre => {
					if let Ok(id) = filter.object.get("id").as_string() {
						let id = id.read();
						match filter.value.as_int().unwrap_or(-1) {
							0 => tags.push(format!("-{}", id)),
							1 => tags.push(id),
							_ => continue,
						}
					}
				}
				FilterType::Check => {
					let value = filter.value.as_int().unwrap_or(-1);
					match filter.name.as_str() {
						"Сортировка по дате" => {
							order_by_date_when_search = value == 1
						}
						_ => continue,
					}
				}
				_ => continue,
			}
		}

		let url = if title.is_empty() && tags.is_empty() {
			format!(
				"{url}/{new}?offset={offset}{sort}",
				url = self.base_url,
				new = self.custom_new_path.unwrap_or("manga/new"),
				offset = (page - 1) * 20,
				sort = sort,
			)
		} else if title.is_empty() {
			format!(
				"{url}/tags/{tags}?offset={offset}{sort}",
				url = self.base_url,
				tags = urlencode(tags.join("+")),
				offset = (page - 1) * 20,
				sort = sort,
			)
		} else {
			format!(
				"{url}/?do=search&subaction=search&search_start={page}&full_search=0&result_from={offset}&result_num=40&story={title}{need_sort_date}",
				url=self.base_url,
				page=page,
				offset=(page - 1) * 40 + 1,
				title=title,
				need_sort_date=if order_by_date_when_search { "&need_sort_date=true" } else { "" },
			)
		};
		let html = Request::new(&url, HttpMethod::Get).html()?;
		self.parse_manga_list(html, if !title.is_empty() { 40 } else { 20 })
	}

	pub fn get_manga_listing(&self, listing: Listing, _: i32) -> Result<MangaPageResult> {
		if &listing.name == "Случайная" {
			let html = Request::new(
				format!("{}/manga/random", self.base_url).as_str(),
				HttpMethod::Get,
			)
			.html()?;
			self.parse_manga_list(html, 10)
		} else {
			Err(AidokuError {
				reason: aidoku::error::AidokuErrorKind::Unimplemented,
			})
		}
	}

	pub fn get_manga_details(&self, id: String) -> Result<Manga> {
		let url = format!("{}{id}", self.base_url);
		cache_manga_page(&url);
		let html = Node::new(unsafe { &CACHED_MANGA.clone().unwrap() })?;
		let cover = html.select("img#cover").attr("src").read();
		let title = html.select("a.title_top_a").text().read();
		let author = html
			.select(self.author_selector)
			.array()
			.filter_map(|elem| {
				let text = elem.as_node().expect("node array").text().read();
				if text.is_empty() {
					None
				} else {
					Some(text)
				}
			})
			.collect::<Vec<_>>()
			.join(", ");
		let description = text_with_newlines(html.select("div#description"));
		let mut categories = html
			.select("li.sidetag a:not([title])")
			.array()
			.map(|elem| elem.as_node().expect("node array").text().read())
			.collect::<Vec<_>>();
		let comictype = html.select("a[href*=type]").text().read();
		if !comictype.is_empty() {
			categories.push(comictype);
		}
		let status_str = html
			.select("table.mangatitle tr:contains(Загружено) td.item2 h2")
			.text()
			.read();
		let status = if status_str.contains("перевод продолжается") {
			MangaStatus::Ongoing
		} else if status_str.contains("перевод завершен") {
			MangaStatus::Completed
		} else {
			MangaStatus::Unknown
		};
		let (nsfw, viewer) = category_parser(&categories);
		Ok(Manga {
			id,
			cover,
			title,
			author: String::from(author.trim()),
			artist: String::new(),
			description,
			url,
			categories,
			status,
			nsfw,
			viewer,
		})
	}

	pub fn get_chapter_list(&self, id: String) -> Result<Vec<Chapter>> {
		cache_manga_page(&format!("{}{id}", self.base_url));
		let html = Node::new(unsafe { &CACHED_MANGA.clone().unwrap() })?;
		let manga_title = html.select("a.title_top_a").text().read();
		let scanlator = html
			.select("a[href*=translation][title]")
			.array()
			.filter_map(|elem| {
				let text = elem.as_node().expect("node array").text().read();
				if text.is_empty() {
					None
				} else {
					Some(text)
				}
			})
			.collect::<Vec<_>>()
			.join(", ");
		let node = html.select("table.table_cha tr[class*=zaliv]");
		Ok(node
			.array()
			.map(|elem| {
				let chapter_node = elem.as_node().expect("node array");
				let id = chapter_node.select("a").attr("href").read();
				let url = format!("{}{id}", self.base_url);
				let date_updated = chapter_node
					.select("div.date")
					.text()
					.0
					.as_date("yyyy-MM-dd", None, None)
					.unwrap_or(-1.0);
				let mut title = chapter_node.select("a").text().read();
				let (volume, chapter) = (self.vol_chap_parser)(manga_title.clone(), title.clone());
				if chapter >= 0.0 {
					let splitter = format!(" {}", chapter);
					if title.contains(&splitter) {
						let split = title.splitn(2, &splitter).collect::<Vec<&str>>();
						title = String::from(split[1]).replacen(
							|char| char == ':' || char == '-',
							"",
							1,
						);
					}
				}
				Chapter {
					id,
					title: String::from(title.trim()),
					volume,
					chapter,
					date_updated,
					scanlator: scanlator.clone(),
					url,
					lang: String::from("ru"),
				}
			})
			.collect::<Vec<_>>())
	}

	pub fn get_page_list(&self, id: String) -> Result<Vec<Page>> {
		let url = if id.starts_with("http") {
			// exhentai-dono.me
			format!("{id}&development_access=true")
		} else {
			format!("{}{id}", self.base_url)
		};
		let html = Request::new(&url, HttpMethod::Get).html()?.html().read();
		let (begin, end) = if let Some(begin_) = html.find("fullimg\":[") {
			// manga-chan and yaoi-chan
			let begin = begin_ + 10;
			let end = html[begin..].find(",]").map(|i| i + begin).unwrap_or(0);
			(begin, end)
		} else if let Some(begin_) = html.find("fullimg\": [") {
			// hentai-chan
			let begin = begin_ + 11;
			let end = html[begin..].find(']').map(|i| i + begin).unwrap_or(0);
			(begin, end)
		} else {
			(0, 2)
		};
		Ok(String::from(&html[begin..end])
			.replace(|ch| ch == '"' || ch == '\'', "")
			.split(',')
			.enumerate()
			.map(|(index, url)| Page {
				index: index as i32,
				url: String::from(url.trim()),
				base64: String::new(),
				text: String::new(),
			})
			.collect::<Vec<_>>())
	}

	pub fn modify_image_request(&self, request: Request) {
		request.header("Referer", self.base_url);
	}

	pub fn handle_url(&self, url: String) -> Result<DeepLink> {
		// https://manga-chan.me/manga/77319-blessed.html
		// ['https:', '', 'manga-chan.me', 'manga', '77319-blessed.html']
		let split = url.split('/').collect::<Vec<_>>();
		if split[3] == "manga" {
			Ok(DeepLink {
				manga: Some(self.get_manga_details(strip_base_url(&url).into())?),
				chapter: None,
			})
		} else if split[3] == "online" {
			let html = Request::new(&url, HttpMethod::Get).html()?.html().read();
			let begin = if let Some(begin_) = html.find("meta\":{") {
				begin_ + 7
			} else if let Some(begin_) = html.find("meta\": {") {
				begin_ + 8
			} else {
				return Err(AidokuError {
					reason: aidoku::error::AidokuErrorKind::Unimplemented,
				});
			};
			let end = html[begin..].find("},").map(|i| i + begin).unwrap_or(0);
			let meta = json::parse(format!("{{{}}}", &html[begin..end - 1]).as_bytes())?;
			let metaobj = meta.as_object()?;
			let manga_id = if let Ok(id) = metaobj.get("content_id").as_string() {
				id.read()
			} else if let Ok(url) = metaobj.get("url").as_string() {
				strip_base_url(&url.read()).into()
			} else {
				return Err(AidokuError {
					reason: aidoku::error::AidokuErrorKind::Unimplemented,
				});
			};
			let manga = Some(self.get_manga_details(manga_id)?);
			let chapter = Some(Chapter {
				id: url.replace(self.base_url, ""),
				title: String::new(),
				volume: -1.0,
				chapter: -1.0,
				date_updated: -1.0,
				scanlator: String::new(),
				url,
				lang: String::from("ru"),
			});
			Ok(DeepLink { manga, chapter })
		} else {
			Err(AidokuError {
				reason: aidoku::error::AidokuErrorKind::Unimplemented,
			})
		}
	}
}
