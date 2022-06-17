use aidoku::{
	error::{AidokuError, Result},
	prelude::*,
	std::{
		html::Node,
		json,
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

use crate::helper::{append_protocol, extract_f32_from_string, text_with_newlines, urlencode};

pub static mut CACHED_MANGA: Option<Node> = None;
static mut CACHED_MANGA_ID: Option<String> = None;

pub fn cache_manga_page(url: &str) {
	unsafe {
		if CACHED_MANGA.is_some() && CACHED_MANGA_ID.clone().unwrap() == url {
			return;
		}

		CACHED_MANGA_ID = Some(String::from(url));
		CACHED_MANGA = Some(Request::new(url, HttpMethod::Get).html());
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

	pub category_parser: fn(&Node, Vec<String>) -> (MangaContentRating, MangaViewer),
	pub category_mapper: fn(i64) -> String,
	pub tags_mapper: fn(i64) -> String,

	pub use_search_engine: bool,
}

impl Default for MMRCMSSource {
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
					format!("{}", idx)
				} else {
					String::new()
				}
			}, // 0 is reserved for None
			tags_mapper: |_| String::new(),
			use_search_engine: true,
		}
	}
}

impl MMRCMSSource {
	fn guess_cover(&self, url: &str, id: &str) -> String {
		if url.ends_with("no-image.png") || url.is_empty() {
			format!(
				"{}/uploads/manga/{}/cover/cover_250x350.jpg",
				self.base_url, id
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
		.html();
		let manga = html
			.select("ul.manga-list a")
			.array()
			.filter_map(|elem| {
				let node = elem.as_node();
				let title = node.text().read();
				if title.to_lowercase().contains(query) {
					let url = node.attr("abs:href").read();
					let id = url.replace(&format!("{}/{}", self.base_url, self.manga_path), "");
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
					title = urlencode(filter.value.as_string()?.read());
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
			if self.use_search_engine {
				let url = format!("{}/search?query={title}", self.base_url);
				if let Ok(json) = Request::new(&url, HttpMethod::Get).json().as_object() {
					let suggestions = json.get("suggestions").as_array()?;
					let mut manga = Vec::with_capacity(suggestions.len());
					for suggestion in suggestions {
						let obj = suggestion.as_object()?;
						let id = obj.get("data").as_string()?.read();
						manga.push(Manga {
							id: id.clone(),
							cover: self.guess_cover("", &id),
							title: obj.get("value").as_string()?.read(),
							url: format!("{}/{}/{}", self.base_url, self.manga_path, id),
							..Default::default()
						});
					}
					Ok(MangaPageResult {
						manga,
						has_more: false,
					})
				} else {
					self.self_search(title)
				}
			} else {
				self.self_search(title)
			}
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
			let has_more: bool = !elems.is_empty();

			for elem in elems {
				let manga_node = elem.as_node();
				let url = manga_node
					.select(&format!("a[href*='{}/{}']", self.base_url, self.manga_path))
					.attr("abs:href")
					.read();
				let id = url.replace(
					format!("{}/{}/", self.base_url, self.manga_path).as_str(),
					"",
				);
				let cover = self.guess_cover(
					&manga_node
						.select(&format!(
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
		let description = text_with_newlines(html.select(".row .well p"));
		let mut manga = Manga {
			id,
			cover,
			title,
			description,
			url,
			..Default::default()
		};

		for elem in html.select(".row .dl-horizontal dt").array() {
			let node = elem.as_node();
			let text = node.text().read().to_lowercase();
			let end = text.find(':').unwrap_or(text.len());
			#[rustfmt::skip]
			match &text[..end] {
				"author(s)" | "autor(es)" | "auteur(s)" | "著作" | "yazar(lar)" | "mangaka(lar)" | "pengarang/penulis" | "pengarang" | "penulis" | "autor" | "المؤلف" | "перевод" | "autor/autorzy" | "автор" => {
					manga.author = node.next().unwrap().text().read()
				}
				"artist(s)" | "artiste(s)" | "sanatçi(lar)" | "artista(s)" | "artist(s)/ilustrator" | "الرسام" | "seniman" | "rysownik/rysownicy" => { 
					manga.artist = node.next().unwrap().text().read()
				}
				"categories" | "categorías" | "catégories" | "ジャンル" | "kategoriler" | "categorias" | "kategorie" | "التصنيفات" | "жанр" | "kategori" | "tagi" | "tags" => {
					node
						.next()
						.unwrap()
						.select("a")
						.array()
						.for_each(|elem| manga.categories.push(elem.as_node().text().read()))
				}
				"status" | "statut" | "estado" | "状態" | "durum" | "الحالة" | "статус" => {
					manga.status = match node.next().unwrap().text().read().to_lowercase().trim() {
						"complete" | "مكتملة" | "complet" | "completo" | "zakończone" | "concluído" => MangaStatus::Completed,
						"ongoing" | "مستمرة" | "en cours" | "em lançamento" | "prace w toku" | "ativo" | "em andamento" | "en curso" => MangaStatus::Ongoing,
						"wstrzymane" => MangaStatus::Hiatus,
						"porzucone" => MangaStatus::Cancelled,
						_ => MangaStatus::Unknown,
					}
				}
				"type" | "ttipo" | "النوع" | "tür" => {
					manga.categories.push(node.next().unwrap().text().read())
				}
				_ => continue,
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
			.map(|elem| {
				let chapter_node = elem.as_node();
				let volume = extract_f32_from_string(
					String::from("volume-"),
					chapter_node.attr("class").read(),
				);
				let url = chapter_node.select("a").attr("abs:href").read();
				let chapter_title = chapter_node.select("a").first().text().read();

				let chapter = extract_f32_from_string(title.clone(), chapter_title.clone());
				let mut title = chapter_node.select("em").text().read();
				if title.is_empty() && should_extract_chapter_title {
					title = chapter_title;
				}

				let chapter_id = String::from(url.split('/').collect::<Vec<_>>()[5]);
				let date_updated = chapter_node
					.select("div.date-chapter-title-rtl, div.col-md-4")
					.first()
					.own_text()
					.as_date("dd MMM'.' yyyy", Some("en_US"), None);
				Chapter {
					id: chapter_id,
					title,
					volume,
					chapter,
					date_updated,
					url,
					lang: String::from(self.lang),
					..Default::default()
				}
			})
			.collect::<Vec<Chapter>>())
	}

	pub fn get_page_list(&self, manga_id: String, id: String) -> Result<Vec<Page>> {
		let url = format!("{}/{}/{}/{}", self.base_url, self.manga_path, manga_id, id);
		let html = Request::new(&url, HttpMethod::Get).html().html().read();
		let begin = html.find("var pages = ").unwrap_or(0) + 12;
		let end = html[begin..].find(';').map(|i| i + begin).unwrap_or(0);
		let array = json::parse(&html[begin..end]).as_array()?;
		let mut pages = Vec::with_capacity(array.len());

		for (idx, page) in array.enumerate() {
			let pageobj = page.as_object()?;
			let url_ = pageobj.get("page_image").as_string()?.read();
			let url = if pageobj.get("external").as_int().unwrap_or(-1) == 0 {
				format!(
					"{}/uploads/manga/{}/chapters/{}/{}",
					self.base_url, manga_id, id, url_
				)
			} else {
				url_
			};
			pages.push(Page {
				index: idx as i32,
				url,
				..Default::default()
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
				let end = split[5].find('-').unwrap_or(split[5].len());
				Some(Chapter {
					id: String::from(split[5]),
					chapter: extract_f32_from_string(String::new(), String::from(&split[5][..end])),
					url: format!(
						"{}/{}/{}/{}",
						self.base_url, self.manga_path, split[4], split[5]
					),
					lang: String::from(self.lang),
					..Default::default()
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
