use aidoku::{
	error::{AidokuError, Result},
	prelude::*,
	std::{
		html::Node,
		json,
		net::{HttpMethod, Request},
		String, StringRef, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

use crate::helper::{append_protocol, extract_f32_from_string, text_with_newlines, urlencode};

static mut CACHED_MANGA: Option<Vec<u8>> = None;
static mut CACHED_MANGA_ID: Option<String> = None;

fn cache_manga_page(url: &str) {
	unsafe {
		if CACHED_MANGA.is_some() && CACHED_MANGA_ID.clone().unwrap() == url {
			return;
		}

		CACHED_MANGA_ID = Some(String::from(url));
		CACHED_MANGA = Some(Request::new(url, HttpMethod::Get).data());
	}
}

pub struct MMRCMSSource {
	pub base_url: &'static str,
	pub lang: &'static str,
	/// {base_url}/{manga_path}/{manga_id}
	pub manga_path: &'static str,

	/// Localization
	pub category: &'static str,
	pub tags: &'static str,

	pub details_title_selector: &'static str,
	pub detail_categories: &'static str,
	pub detail_tags: &'static str,
	pub detail_description: &'static str,
	pub detail_status_ongoing: &'static str,
	pub detail_status_complete: &'static str,

	pub category_parser: fn(&Node, Vec<String>) -> (MangaContentRating, MangaViewer),
	pub category_mapper: fn(i64) -> String,
	pub tags_mapper: fn(i64) -> String,
}

impl Default for MMRCMSSource {
	fn default() -> Self {
		MMRCMSSource {
			base_url: "",
			lang: "en",
			manga_path: "manga",

			category: "Category",
			tags: "Tag",

			details_title_selector: "div.col-sm-12 h2",
			detail_categories: "Categories",
			detail_tags: "Tags",
			detail_description: "Summary",
			detail_status_complete: "Complete",
			detail_status_ongoing: "Ongoing",

			category_parser: |_, categories| {
				let mut nsfw = MangaContentRating::Safe;
				let mut viewer = MangaViewer::Rtl;
				for category in categories {
					match category.as_str() {
						"Adult" | "Smut" | "Mature" | "18+" | "Hentai" => nsfw = MangaContentRating::Nsfw,
						"Ecchi" | "16+" => {
							nsfw = match nsfw {
								MangaContentRating::Nsfw => MangaContentRating::Nsfw,
								_ => MangaContentRating::Suggestive,
							}
						}
						"Webtoon" | "Manhwa" | "Manhua" => viewer = MangaViewer::Scroll,
						_ => continue,
					}
				}
				(nsfw, viewer)
			},
			category_mapper: |idx| {
				if idx != 0 {
					format!("{}", idx)
				} else {
					String::new()
				}
			}, // 0 is reserved for None
			tags_mapper: |_| String::new(),
		}
	}
}

impl MMRCMSSource {
	pub fn get_manga_list(&self, filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
		let mut query: Vec<String> = Vec::new();
		let mut is_searching = false;
		for filter in filters {
			match filter.kind {
				FilterType::Title => {
					is_searching = true;
					query.push(format!(
						"query={}",
						urlencode(filter.value.as_string()?.read())
					));
					break;
				}
				FilterType::Author => {
					query.push(format!(
						"artist={}",
						urlencode(filter.value.as_string()?.read())
					));
				}
				FilterType::Sort => {
					if let Ok(value) = filter.value.as_object() {
						let index = value.get("index").as_int().unwrap_or(0);
						let asc = value.get("ascending").as_bool().unwrap_or(false);
						query.push(String::from(match index {
							0 => "sortBy=name",
							1 => "sortBy=views",
							2 => "sortBy=last_release", // Readcomicsonline.ru
							_ => continue,
						}));
						query.push(format!("asc={}", asc));
					}
				}
				FilterType::Select => {
					let value = filter.value.as_int().unwrap_or(-1);
					if filter.name.as_str() == self.category {
						query.push(format!("cat={}", (self.category_mapper)(value)))
					} else if filter.name.as_str() == self.tags {
						query.push(format!("tag={}", (self.tags_mapper)(value)))
					} else {
						continue;
					}
				}
				_ => continue,
			}
		}
		if is_searching {
			let url = format!("{}/search?{}", self.base_url, query.join("&"));
			let json = Request::new(&url, HttpMethod::Get).json().as_object()?;
			let suggestions = json.get("suggestions").as_array()?;
			let mut manga = Vec::with_capacity(suggestions.len());
			for suggestion in suggestions {
				let obj = suggestion.as_object()?;
				let id = obj.get("data").as_string()?.read();
				manga.push(Manga {
					id: id.clone(),
					cover: format!(
						"{}/uploads/manga/{}/cover/cover_250x350.jpg",
						self.base_url, id
					),
					title: obj.get("value").as_string()?.read(),
					author: String::new(),
					artist: String::new(),
					description: String::new(),
					url: format!("{}/{}/{}", self.base_url, self.manga_path, id),
					categories: Vec::new(),
					status: MangaStatus::Unknown,
					nsfw: MangaContentRating::Safe,
					viewer: MangaViewer::Rtl,
				});
			}
			Ok(MangaPageResult {
				manga,
				has_more: false,
			})
		} else {
			let url = format!(
				"{}/filterList?page={}&{}",
				self.base_url,
				page,
				query.join("&")
			);
			let html = Request::new(&url, HttpMethod::Get).html();
			let node = html.select("div[class^=col-sm-]");
			let elems = node.array();
			let mut manga = Vec::with_capacity(elems.len());
			let has_more: bool = elems.len() > 0;

			for elem in elems {
				let manga_node = elem.as_node();
				let url = manga_node
					.select(&format!("a[href*='{}/{}']", self.base_url, self.manga_path))
					.attr("href")
					.read();
				let id = url.replace(
					format!("{}/{}/", self.base_url, self.manga_path).as_str(),
					"",
				);
				let mut cover_src = manga_node
					.select(&format!(
						"a[href*='{}/{}'] img",
						self.base_url, self.manga_path
					))
					.attr("src")
					.read();
				if cover_src.starts_with('/') && !cover_src.starts_with("//") {
					cover_src = format!("{}{}", self.base_url, cover_src);
				}
				let cover = append_protocol(cover_src);
				let title = manga_node.select("a.chart-title strong").text().read();
				manga.push(Manga {
					id: id.clone(),
					cover,
					title,
					author: String::new(),
					artist: String::new(),
					description: String::new(),
					url,
					categories: Vec::new(),
					status: MangaStatus::Unknown,
					nsfw: MangaContentRating::Safe,
					viewer: MangaViewer::Rtl,
				});
			}
			Ok(MangaPageResult { manga, has_more })
		}
	}

	pub fn get_manga_details(&self, id: String) -> Result<Manga> {
		let url = format!("{}/{}/{}", self.base_url, self.manga_path, id);
		cache_manga_page(&url);
		let html = Node::new(unsafe { &CACHED_MANGA.clone().unwrap() });
		let mut cover_src = html.select("img[class^=img-]").attr("src").read();
		if cover_src.starts_with('/') && !cover_src.starts_with("//") {
			cover_src = format!("{}{}", self.base_url, cover_src);
		}
		let cover = append_protocol(cover_src);
		let title = html
			.select(self.details_title_selector)
			.array()
			.get(0)
			.as_node()
			.text()
			.read();
		let author = html
			.select("a[href*=author]")
			.array()
			.filter_map(|elem| {
				let text = elem.as_node().text().read();
				if text.trim().is_empty() {
					None
				} else {
					Some(String::from(text.trim()))
				}
			})
			.collect::<Vec<_>>()
			.join(", ");
		let artist = html
			.select("a[href*=artist]")
			.array()
			.filter_map(|elem| {
				let text = elem.as_node().text().read();
				if text.trim().is_empty() {
					None
				} else {
					Some(String::from(text.trim()))
				}
			})
			.collect::<Vec<_>>()
			.join(", ");
		let description = text_with_newlines(
			html.select(format!("div:contains({}) p", self.detail_description).as_str()),
		);

		let mut categories = html
			.select(&format!("dt:contains({}) + dd a", self.detail_categories))
			.array()
			.chain(
				html.select(&format!("dt:contains({}) + dd a", self.detail_tags))
					.array(),
			)
			.filter_map(|elem| {
				let text = elem.as_node().text().read();
				if text.trim().is_empty() {
					None
				} else {
					Some(String::from(text.trim()))
				}
			})
			.collect::<Vec<_>>();
		if categories.is_empty() {
			// Fallback fetcher
			categories = html
				.select(
					&format!("a[href*={}][href~=(?i)(tag|category)]:not([target=_blank])", self.base_url)
				)
				.array()
				.filter_map(|elem| {
					let text = elem.as_node().text().read();
					if text.trim().is_empty() {
						None
					} else {
						Some(String::from(text.trim()))
					}
				})
				.collect::<Vec<_>>();
		}
		categories.sort_unstable();
		categories.dedup();
		let (mut nsfw, viewer) = (self.category_parser)(&html, categories.clone());
		if html.select("div.alert.alert-danger").array().len() > 0 {
			nsfw = MangaContentRating::Nsfw;
		}
		let status_str = html.select("span[class^='label label-']").text().read();
		let status = if status_str.trim() == self.detail_status_complete {
			MangaStatus::Completed
		} else if status_str.trim() == self.detail_status_ongoing {
			MangaStatus::Ongoing
		} else {
			MangaStatus::Unknown
		};
		Ok(Manga {
			id,
			cover,
			title,
			author,
			artist,
			description,
			url,
			categories,
			status,
			nsfw,
			viewer,
		})
	}

	pub fn get_chapter_list(&self, id: String) -> Result<Vec<Chapter>> {
		let url = format!("{}/{}/{}", self.base_url, self.manga_path, id);
		cache_manga_page(&url);
		let html = Node::new(unsafe { &CACHED_MANGA.clone().unwrap() });
		let node = html.select("li:has(h5.chapter-title-rtl)");
		let elems = node.array();
		let title = html
			.select(self.details_title_selector)
			.array()
			.get(0)
			.as_node()
			.text()
			.read();
		Ok(elems
			.map(|elem| {
				let chapter_node = elem.as_node();
				let volume = extract_f32_from_string(
					String::from("volume-"),
					chapter_node.attr("class").read(),
				);
				let url = chapter_node.select("a").attr("href").read();
				let chapter_title = chapter_node.select("a").text().read();

				let chapter = extract_f32_from_string(
					title.clone(),
					chapter_title,
				);
				let title = chapter_node.select("em").text().read();
				let chapter_id = format!("{}/{}", id, url.split('/').collect::<Vec<_>>()[5]);
				let date_updated = StringRef::from(
					chapter_node
						.select("div.date-chapter-title-rtl")
						.text()
						.read()
						.trim(),
				)
				.0
				.as_date("dd MMM'.' yyyy", Some("en_US"), None)
				.unwrap_or(-1.0);
				Chapter {
					id: chapter_id,
					title,
					volume,
					chapter,
					date_updated,
					scanlator: String::new(),
					url,
					lang: String::from(self.lang),
				}
			})
			.collect::<Vec<Chapter>>())
	}

	pub fn get_page_list(&self, id: String) -> Result<Vec<Page>> {
		let url = format!("{}/{}/{}", self.base_url, self.manga_path, id);
		let html = Request::new(&url, HttpMethod::Get).html().html().read();
		let begin = html.find("var pages = ").unwrap_or(0) + 12;
		let end = html[begin..].find(';').map(|i| i + begin).unwrap_or(0);
		let array = json::parse(html[begin..end].as_bytes()).as_array()?;
		let mut pages = Vec::with_capacity(array.len());

		let (manga_id, chapter_id) = {
			let split = id.split('/').collect::<Vec<_>>();
			(split[0], split[1])
		};
		for (idx, page) in array.enumerate() {
			let pageobj = page.as_object()?;
			let url_ = pageobj.get("page_image").as_string()?.read();
			let url = if pageobj.get("external").as_int().unwrap_or(-1) == 0 {
				format!(
					"{}/uploads/manga/{}/chapters/{}/{}",
					self.base_url, manga_id, chapter_id, url_
				)
			} else {
				url_
			};
			pages.push(Page {
				index: idx as i32,
				url,
				base64: String::new(),
				text: String::new(),
			});
		}
		Ok(pages)
	}

	pub fn modify_image_request(&self, request: Request) {
		request.header("Referer", self.base_url);
	}

	pub fn handle_url(&self, url: String) -> Result<DeepLink> {
		// https://manga.fascans.com/manga/aharensan-wa-hakarenai/11/1
		// ['https:', '', 'manga.fascans.com', 'manga', 'aharensan-wa-hakarenai', '11',
		// '1']
		let split = url.split('/').collect::<Vec<_>>();
		if split.len() > 4 {
			let manga = Some(self.get_manga_details(String::from(split[4]))?);
			let chapter = if split.len() > 5 {
				let id = format!("{}/{}", split[4], split[5]);
				Some(Chapter {
					id: id.clone(),
					title: String::new(),
					volume: -1.0,
					chapter: extract_f32_from_string(String::new(), String::from(split[5])),
					date_updated: -1.0,
					scanlator: String::new(),
					url: format!("{}/{}/{}", self.base_url, self.manga_path, id),
					lang: String::from(self.lang),
				})
			} else {
				None
			};
			Ok(DeepLink { manga, chapter })
		} else {
			Err(AidokuError {
				reason: aidoku::error::AidokuErrorKind::Unimplemented,
			})
		}
	}
}
