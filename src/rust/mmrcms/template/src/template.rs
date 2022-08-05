use aidoku::{
	error::{AidokuError, Result},
	helpers::{cfemail::decode_cfemail, substring::Substring, uri::encode_uri_component},
	prelude::format,
	std::{
		html::Node,
		json,
		net::{HttpMethod, Request},
		ObjectRef, String, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

use crate::helper::{append_protocol, extract_f32_from_string};

pub static mut CACHED_MANGA: Option<Node> = None;
static mut CACHED_MANGA_ID: Option<String> = None;

/// Internal attribute to control if the source should fall
/// back to self searching after failing to use the search
/// engine first time.
///
/// Strikes a balance between control and reliability (and also
/// not spamming sources with useless requests)
static mut INTERNAL_USE_SEARCH_ENGINE: bool = true;

pub fn cache_manga_page(url: &str) {
	unsafe {
		if CACHED_MANGA.is_some() && CACHED_MANGA_ID.clone().unwrap_or_default() == url {
			return;
		}

		if let Ok(html) = Request::new(url, HttpMethod::Get).html() {
			decode_cfemail(&html);
			CACHED_MANGA = Some(html);
			CACHED_MANGA_ID = Some(String::from(url));
		}
	}
}

pub struct MMRCMSSource<'a> {
	pub base_url: &'a str,
	pub lang: &'a str,
	/// {base_url}/{manga_path}/{manga_id}
	pub manga_path: &'a str,

	/// Localization
	pub category: &'a str,
	pub tags: &'a str,

	pub category_parser: fn(&Node, Vec<String>) -> (MangaContentRating, MangaViewer),
	pub category_mapper: fn(i64) -> String,
	pub tags_mapper: fn(i64) -> String,

	pub use_search_engine: bool,
}

#[derive(Default)]
struct MMRCMSSearchResult {
	pub data: String,
	pub value: String,
}

impl TryFrom<ObjectRef> for MMRCMSSearchResult {
	type Error = AidokuError;

	fn try_from(obj: ObjectRef) -> Result<Self> {
		Ok(Self {
			data: obj.get("data").as_string()?.read(),
			value: obj.get("value").as_string()?.read(),
		})
	}
}

impl<'a> Default for MMRCMSSource<'a> {
	fn default() -> Self {
		MMRCMSSource {
			base_url: "",
			lang: "en",
			manga_path: "manga",

			category: "Category",
			tags: "Tag",

			category_parser: |_, categories| {
				let mut nsfw = MangaContentRating::Safe;
				let mut viewer = MangaViewer::Rtl;
				for category in categories {
					match category.as_str() {
						"Adult" | "Smut" | "Mature" | "18+" | "Hentai" => {
							nsfw = MangaContentRating::Nsfw
						}
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
					String::from(itoa::Buffer::new().format(idx))
				} else {
					String::new()
				}
			}, // 0 is reserved for None
			tags_mapper: |_| String::new(),
			use_search_engine: true,
		}
	}
}

impl<'a> MMRCMSSource<'a> {
	fn guess_cover(&self, url: &str, id: &str) -> String {
		if url.ends_with("no-image.png") || url.is_empty() {
			format!(
				"{base_url}/uploads/manga/{id}/cover/cover_250x350.jpg",
				base_url = self.base_url
			)
		} else {
			append_protocol(String::from(url))
		}
	}

	fn self_search<T: AsRef<str>>(&self, query: T) -> Result<MangaPageResult> {
		let query = query.as_ref();
		let html = Request::new(
			format!("{}/changeMangaList?type=text", self.base_url),
			HttpMethod::Get,
		)
		.html()?;
		decode_cfemail(&html);
		let manga = html
			.select("ul.manga-list a")
			.array()
			.filter_map(|elem| {
				if let Ok(node) = elem.as_node()
				   && let Ok(title) = elem.as_node().map(|v| v.text().read())
				   && title.to_lowercase().contains(query) {
					let url = node.attr("abs:href").read();
					let id = url
						.split('/')
						.last()
						.map(String::from)
						.unwrap_or_else(|| url.replace(&format!("{}/{}", self.base_url, self.manga_path), ""));
					let cover = self.guess_cover("", &id);
					Some(Manga {
						id,
						cover,
						title,
						url,
						..Default::default()
					})
				} else {
					None
				}
			})
			.collect::<Vec<_>>();

		Ok(MangaPageResult {
			manga,
			has_more: false,
		})
	}

	pub fn get_manga_list(&self, filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
		let mut query: Vec<String> = Vec::new();
		let mut title = String::new();
		for filter in filters {
			match filter.kind {
				FilterType::Title => {
					let t = filter
						.value
						.as_string()
						.map(|v| v.read())
						.unwrap_or_default();
					if t.is_empty() {
						continue;
					}
					title = encode_uri_component(t);
					break;
				}
				FilterType::Author => query.push(format!(
					"artist={}",
					encode_uri_component(
						filter
							.value
							.as_string()
							.map(|v| v.read())
							.unwrap_or_default()
					)
				)),
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
						query.push(format!("asc={}", if asc { "true" } else { "false" }));
					}
				}
				FilterType::Select => {
					let value = filter.value.as_int().unwrap_or(-1);
					match filter.name.as_str() {
						x if x == self.category => {
							query.push(format!("cat={}", (self.category_mapper)(value)))
						}
						x if x == self.tags => {
							query.push(format!("tag={}", (self.tags_mapper)(value)))
						}
						_ => continue,
					}
				}
				_ => continue,
			}
		}
		if !title.is_empty() {
			if self.use_search_engine && unsafe { INTERNAL_USE_SEARCH_ENGINE } {
				let url = format!("{}/search?query={}", self.base_url, title);
				if let Ok(obj) = Request::new(&url, HttpMethod::Get).json()
				   && let Ok(json) = obj.as_object()
				   && let Ok(suggestions) = json.get("suggestions").as_array() {
					let mut manga = Vec::with_capacity(suggestions.len());
					for suggestion in suggestions {
						if let Ok(suggestion) = suggestion.as_object()
						   && let Ok(obj) = MMRCMSSearchResult::try_from(suggestion) {
							manga.push(Manga {
								cover: self.guess_cover("", &obj.data),
								url: format!("{}/{}/{}", self.base_url, self.manga_path, obj.data),
								id: obj.data,
								title: obj.value,
								..Default::default()
							});
						}
					}
					Ok(MangaPageResult {
						manga,
						has_more: false,
					})
				} else {
					unsafe { INTERNAL_USE_SEARCH_ENGINE = false };
					self.self_search(title)
				}
			} else {
				self.self_search(title)
			}
		} else {
			let url = format!(
				"{}/filterList?page={}&{}",
				self.base_url,
				itoa::Buffer::new().format(page),
				query.join("&")
			);
			let html = Request::new(&url, HttpMethod::Get).html()?;
			decode_cfemail(&html);
			let node = html.select("div[class^=col-sm-]");
			let elems = node.array();
			let mut manga = Vec::with_capacity(elems.len());
			let has_more: bool = !elems.is_empty();

			for elem in elems {
				if let Ok(manga_node) = elem.as_node() {
					let url = manga_node
						.select(format!("a[href*='{}/{}']", self.base_url, self.manga_path))
						.attr("abs:href")
						.read();
					let id = url.replace(&format!("{}/{}/", self.base_url, self.manga_path), "");
					let cover = self.guess_cover(
						&manga_node
							.select(format!(
								"a[href*='{}/{}'] img",
								self.base_url, self.manga_path
							))
							.attr("abs:src")
							.read(),
						&id,
					);
					let title = manga_node.select("a.chart-title strong").text().read();
					manga.push(Manga {
						id: id.clone(),
						cover,
						title,
						url,
						..Default::default()
					});
				}
			}
			Ok(MangaPageResult { manga, has_more })
		}
	}

	pub fn get_manga_details(&self, id: String) -> Result<Manga> {
		let url = format!("{}/{}/{}", self.base_url, self.manga_path, id);
		cache_manga_page(&url);
		let html = unsafe { CACHED_MANGA.clone().unwrap() };
		let cover = append_protocol(html.select("img[class^=img-]").attr("abs:src").read());
		let title = html
			.select("h2.widget-title, h1.widget-title, .listmanga-header, div.panel-heading")
			.first()
			.text()
			.read();
		let description = html.select(".row .well p").untrimmed_text().read();
		let mut manga = Manga {
			id,
			cover,
			title,
			description,
			url,
			..Default::default()
		};

		for elem in html.select(".row .dl-horizontal dt").array() {
			if let Ok(node) = elem.as_node()
			   && let Some(next_node) = node.next() {
				let text = node.text().read().to_lowercase();
				#[rustfmt::skip]
				match text.substring_before(':').unwrap_or(&text) {
					"author(s)" | "autor(es)" | "auteur(s)" | "著作" | "yazar(lar)" | "mangaka(lar)" | "pengarang/penulis" | "pengarang" | "penulis" | "autor" | "المؤلف" | "перевод" | "autor/autorzy" | "автор" => {
						manga.author = next_node.text().read();
					}
					"artist(s)" | "artiste(s)" | "sanatçi(lar)" | "artista(s)" | "artist(s)/ilustrator" | "الرسام" | "seniman" | "rysownik/rysownicy" => { 
						manga.artist = next_node.text().read()
					}
					"categories" | "categorías" | "catégories" | "ジャンル" | "kategoriler" | "categorias" | "kategorie" | "التصنيفات" | "жанр" | "kategori" | "tagi" | "tags" => {
						manga.categories.extend(next_node
							.select("a")
							.array()
							.filter_map(|elem| elem.as_node().map(|node| node.text().read()).ok())
						)
					}
					"status" | "statut" | "estado" | "状態" | "durum" | "الحالة" | "статус" => {
						manga.status = match next_node.text().read().to_lowercase().trim() {
							"complete" | "مكتملة" | "complet" | "completo" | "zakończone" | "concluído" => MangaStatus::Completed,
							"ongoing" | "مستمرة" | "en cours" | "em lançamento" | "prace w toku" | "ativo" | "em andamento" | "en curso" => MangaStatus::Ongoing,
							"wstrzymane" => MangaStatus::Hiatus,
							"porzucone" => MangaStatus::Cancelled,
							_ => MangaStatus::Unknown,
						}
					}
					"type" | "ttipo" | "النوع" | "tür" => {
						manga.categories.push(next_node.text().read())
					}
					_ => continue,
				}
			}
		}
		manga.categories.sort_unstable();
		manga.categories.dedup();
		(manga.nsfw, manga.viewer) = (self.category_parser)(&html, manga.categories.clone());
		if !html.select("div.alert.alert-danger").array().is_empty() {
			manga.nsfw = MangaContentRating::Nsfw;
		}
		Ok(manga)
	}

	pub fn get_chapter_list(&self, id: String) -> Result<Vec<Chapter>> {
		let url = format!("{}/{}/{}", self.base_url, self.manga_path, id);
		cache_manga_page(&url);
		let html = unsafe { CACHED_MANGA.clone().unwrap() };
		let node = html.select("li:has(.chapter-title-rtl)");
		let elems = node.array();
		let title = html
			.select("h2.widget-title, h1.widget-title, .listmanga-header, div.panel-heading")
			.first()
			.text()
			.read();
		let should_extract_chapter_title = node.select("em").array().is_empty();
		Ok(elems
			.filter_map(|elem| {
				if let Ok(chapter_node) = elem.as_node() {
					let url = chapter_node.select("a").attr("abs:href").read();

					if let Some(chapter_id) = url.split('/').nth(5).map(String::from) {
						let volume = extract_f32_from_string(
							String::from("volume-"),
							chapter_node.attr("class").read(),
						);
						let chapter_title = chapter_node.select("a").first().text().read();

						let chapter = extract_f32_from_string(title.clone(), chapter_title.clone());
						let mut title = chapter_node.select("em").text().read();
						if title.is_empty() && should_extract_chapter_title {
							title = chapter_title;
						}

						let date_updated = chapter_node
							.select("div.date-chapter-title-rtl, div.col-md-4")
							.first()
							.own_text()
							.as_date("dd MMM'.' yyyy", Some("en_US"), None);

						Some(Chapter {
							id: chapter_id,
							title,
							volume,
							chapter,
							date_updated,
							url,
							lang: String::from(self.lang),
							..Default::default()
						})
					} else {
						None
					}
				} else {
					None
				}
			})
			.collect::<Vec<Chapter>>())
	}

	pub fn get_page_list(&self, manga_id: String, id: String) -> Result<Vec<Page>> {
		let url = format!("{}/{}/{}/{}", self.base_url, self.manga_path, manga_id, id);
		let html = Request::new(&url, HttpMethod::Get).string()?;
		let array = json::parse(
			html.substring_after("var pages = ")
				.unwrap_or_default()
				.substring_before(";")
				.unwrap_or_default(),
		)?
		.as_array()?;
		let mut pages = Vec::with_capacity(array.len());

		for (idx, page) in array.enumerate() {
			if let Ok(pageobj) = page.as_object()
			   && let Ok(page_image) = pageobj.get("page_image").as_string() {
				let page_image = page_image.read();
				let url = if pageobj.get("external").as_int().unwrap_or(-1) == 0 {
					format!("{}/uploads/manga/{}/chapters/{}/{}", self.base_url, manga_id, id, page_image)
				} else {
					page_image
				};
				pages.push(Page {
					index: idx as i32,
					url,
					..Default::default()
				});
			}
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
		let mut split = url.split('/');
		if let Some(id) = split.nth(4).map(String::from) {
			let manga = Some(self.get_manga_details(id)?);
			let chapter = split.next().map(String::from).map(|id| Chapter {
				id,
				..Default::default()
			});
			Ok(DeepLink { manga, chapter })
		} else {
			Err(AidokuError {
				reason: aidoku::error::AidokuErrorKind::Unimplemented,
			})
		}
	}
}
