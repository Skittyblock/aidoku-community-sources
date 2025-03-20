use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	prelude::format,
	std::json::parse,
	std::net::{HttpMethod, Request},
	std::{String, Vec},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaViewer,
	Page,
};

use crate::helper::*;

pub const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/604";

pub struct MangaStreamSource {
	/// Use static post ids instead of dynamic ids parsed from urls \
	/// Cannot be used together with `has_permanent_manga_url` or
	/// `has_permanent_chapter_url`
	pub use_manga_postids: bool,
	pub use_chapter_postids: bool,
	pub has_permanent_manga_url: bool,
	pub has_permanent_chapter_url: bool,
	pub has_random_chapter_prefix: bool,
	pub is_nsfw: bool,
	pub tagid_mapping: fn(String) -> String,
	pub listing: [&'static str; 3],
	pub base_url: String,
	pub traverse_pathname: &'static str,
	pub directory_pathname: Option<&'static str>,
	pub next_page: &'static str,
	pub next_page_2: &'static str,
	pub manga_selector: &'static str,
	pub manga_title: &'static str,
	pub manga_title_trim: Vec<String>,
	pub last_page_text: &'static str,
	pub last_page_text_2: &'static str,
	pub status_options: [&'static str; 5],
	pub status_options_2: [&'static str; 5],

	pub manga_details_categories: &'static str,
	pub nsfw_genres: Vec<String>,
	pub manga_details_title: &'static str,
	pub manga_details_cover: &'static str,
	pub manga_details_cover_src: &'static str,
	pub manga_details_author: &'static str,
	pub manga_details_artist: &'static str,
	pub manga_details_description: &'static str,
	pub manga_details_status: &'static str,
	pub manga_details_type: &'static str,
	pub manga_details_type_options: &'static str,

	pub chapter_selector: &'static str,
	pub chapter_title: &'static str,
	pub chapter_date: &'static str,
	pub chapter_url: &'static str,
	pub chapter_date_format: &'static str,
	pub chapter_date_format_2: &'static str,
	pub language: &'static str,
	pub language_2: &'static str,
	pub locale: &'static str,
	pub locale_2: &'static str,
	pub date_string: &'static str,

	pub alt_pages: bool,
	pub page_selector: &'static str,
	pub page_url: &'static str,
	pub protocol: bool,
}
impl Default for MangaStreamSource {
	fn default() -> Self {
		MangaStreamSource {
			use_manga_postids: false,
			use_chapter_postids: false,
			has_permanent_manga_url: false,
			has_permanent_chapter_url: false,
			// this is for urls like https://mangashit.cum/RANDOM_INT_PREFIX/chapter-1
			has_random_chapter_prefix: false,
			is_nsfw: false,
			tagid_mapping: |str| str,
			listing: ["Latest", "Popular", "New"],
			base_url: String::new(),
			traverse_pathname: "manga",
			directory_pathname: None,
			next_page: ".hpage a.r",
			next_page_2: ".hpage a.r",
			manga_selector: ".listupd .bsx",
			manga_title: "a",
			manga_title_trim: ["light novel".into()].to_vec(),
			last_page_text: "Next",
			last_page_text_2: "NNNN",
			status_options: ["Ongoing", "Completed", "Hiatus", "Cancelled", "Dropped"],
			status_options_2: ["","","","",""],

			manga_details_categories: "span.mgen a",
			nsfw_genres: [ "Adult".into(), "Ecchi".into(), "Mature".into(), "Smut".into() ].to_vec(),
			manga_details_title: "h1.entry-title",
			manga_details_cover: ".infomanga > div[itemprop=image] img, .thumb img",
			manga_details_cover_src: "src",
			manga_details_author: "span:contains(Author:), span:contains(Pengarang:), .fmed b:contains(Author)+span, .imptdt:contains(Author) i, .fmed b:contains(Yazar)+span, .fmed b:contains(Autheur)+span",
			manga_details_artist: "#last_episode small",
			manga_details_description: "div.desc p, div.entry-content p, div[itemprop=description]:not(:has(p))",
			manga_details_status: ".imptdt:contains(Status), .imptdt:contains(Durum), .imptdt:contains(Statut) i",
			manga_details_type: ".imptdt a",
			manga_details_type_options: "Manga",

			chapter_selector: "#chapterlist li",
			chapter_title: "span.chapternum",
			chapter_date: "span.chapterdate",
			chapter_url: "a",
			date_string: "NNNN",
			chapter_date_format : "MMM dd, yyyy",
			chapter_date_format_2: "",
			language: "en",
			language_2: "",
			locale: "en_US",
			locale_2: "",

			alt_pages: false,
			page_selector: "#readerarea img",
			page_url: "src",
			protocol: false
		}
	}
}

impl MangaStreamSource {
	// parse the homepage and filters
	pub fn parse_manga_list(&self, filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
		let mut included_tags: Vec<String> = Vec::new();
		let mut excluded_tags: Vec<String> = Vec::new();
		let mut status: String = String::new();
		let mut title: String = String::new();
		let mut manga_type: String = String::new();
		let status_options = ["", "ongoing", "completed", "hiatus"];
		let type_options = ["", "manga", "manhwa", "manhua", "comic"];
		for filter in filters {
			match filter.kind {
				FilterType::Title => {
					let input = filter.value.as_string()?.read();
					// replace characters that mess up search with spaces
					// without using spaces search will possibly not find accurate matches
					title = input.replace(['’', '‘', '“', '”', '–'], " ");
				}
				FilterType::Genre => match filter.value.as_int().unwrap_or(-1) {
					0 => match !self.language_2.is_empty() {
						true => excluded_tags.push((self.tagid_mapping)(filter.name)),
						_ => excluded_tags.push(filter.object.get("id").as_string()?.read()),
					},
					1 => match !self.language_2.is_empty() {
						true => included_tags.push((self.tagid_mapping)(filter.name)),
						_ => included_tags.push(filter.object.get("id").as_string()?.read()),
					},
					_ => continue,
				},

				FilterType::Select => {
					let index = filter.value.as_int().unwrap_or(-1) as usize;
					match filter.name.as_str() {
						"Status" => status = String::from(status_options[index]),
						"Type" => manga_type = String::from(type_options[index]),
						_ => continue,
					}
				}
				_ => continue,
			};
		}
		let url = get_search_url(
			self,
			title,
			page,
			included_tags,
			excluded_tags,
			status,
			manga_type,
		);
		self.parse_manga_listing(url, String::from("Latest"), page)
	}

	// parse the listing page (popular, latest , new etc)
	pub fn parse_manga_listing(
		&self,
		base_url: String,
		listing_name: String,
		page: i32,
	) -> Result<MangaPageResult> {
		let url = if base_url == self.base_url {
			get_listing_url(
				self.listing,
				base_url,
				String::from(self.directory_pathname.unwrap_or(self.traverse_pathname)),
				listing_name,
				page,
			)
		} else {
			base_url
		};
		let mut mangas: Vec<Manga> = Vec::new();
		let html = Request::new(url, HttpMethod::Get)
			.header("User-Agent", USER_AGENT)
			.html()?;
		for manga in html.select(self.manga_selector).array() {
			let manga_node = manga.as_node().expect("Failed to get manga as node");
			let title = manga_node.select(self.manga_title).attr("title").read();
			if self
				.manga_title_trim
				.iter()
				.any(|i| title.to_lowercase().contains(i))
			{
				continue;
			}

			let url: String;
			let id: String;

			if self.use_manga_postids {
				let original_url = manga_node.select("a").attr("href").read();
				id = get_postid_from_manga_url(
					original_url,
					&self.base_url,
					self.traverse_pathname,
				)?;
				url = format!("{}/{}/?p={}", self.base_url, self.traverse_pathname, id);
			} else {
				url = {
					let original_url = manga_node.select("a").attr("href").read();

					if self.has_permanent_manga_url {
						get_permanet_url(original_url)
					} else {
						original_url
					}
				};

				id = get_id_from_url(url.clone());
			}

			let cover = get_image_src(manga_node);

			mangas.push(Manga {
				id,
				cover,
				title,
				url,
				..Default::default()
			});
		}
		let last_page_string = if !html.select(self.next_page).text().read().is_empty() {
			html.select(self.next_page).text().read()
		} else {
			html.select(self.next_page_2).text().read()
		};
		let has_more = last_page_string.contains(self.last_page_text)
			|| last_page_string.contains(self.last_page_text_2);
		Ok(MangaPageResult {
			manga: mangas,
			has_more,
		})
	}

	// parse manga details page
	pub fn parse_manga_details(&self, id: String) -> Result<Manga> {
		let url = if self.use_manga_postids {
			format!("{}/{}/?p={}", self.base_url, self.traverse_pathname, id)
		} else {
			format!("{}/{}/{}", self.base_url, self.traverse_pathname, id)
		};
		let html = Request::new(&url, HttpMethod::Get)
			.header("User-Agent", USER_AGENT)
			.html()?;
		let mut title = html.select(self.manga_details_title).text().read();
		for i in self.manga_title_trim.iter() {
			if title.contains(i) {
				title = title.replace(i, "");
			}
		}
		let cover: String = html
			.select(self.manga_details_cover)
			.first()
			.attr(self.manga_details_cover_src)
			.read()
			.replace("?resize=165,225", "");

		let mut author = String::from(
			html.select(self.manga_details_author)
				.first()
				.text()
				.read()
				.replace("[Add, ]", "")
				.replace("Author", "")
				.trim(),
		);
		if author == "-" {
			author = String::new();
		}
		let artist = html.select(self.manga_details_artist).text().read();
		let description = text_with_newlines(html.select(self.manga_details_description));
		let status = manga_status(
			String::from(html.select(self.manga_details_status).text().read().trim()),
			self.status_options,
			self.status_options_2,
		);
		let mut categories = Vec::new();
		let mut nsfw = if self.is_nsfw {
			MangaContentRating::Nsfw
		} else {
			MangaContentRating::Safe
		};
		for node in html.select(self.manga_details_categories).array() {
			let category = node
				.as_node()
				.expect("Failed to get category as node")
				.text()
				.read();
			for genre in self.nsfw_genres.iter() {
				if *genre == category {
					nsfw = MangaContentRating::Nsfw
				}
			}
			categories.push(category.clone());
		}
		let manga_type = html.select(self.manga_details_type).text().read();
		let viewer = if manga_type.as_str() == self.manga_details_type_options {
			MangaViewer::Rtl
		} else {
			MangaViewer::Scroll
		};

		Ok(Manga {
			id,
			cover: append_protocol(cover),
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

	// parse the chapters list present on manga details page
	pub fn parse_chapter_list(&self, id: String) -> Result<Vec<Chapter>> {
		let chapter_url_to_postid_mapping = if self.use_chapter_postids {
			generate_chapter_url_to_postid_mapping(id.clone(), &self.base_url)?
		} else {
			Default::default()
		};

		let url = {
			// yes this should be `use_manga_postids` and not `use_chapter_postids`
			if self.use_manga_postids {
				format!("{}/{}/?p={}", self.base_url, self.traverse_pathname, id)
			} else {
				format!("{}/{}/{}", self.base_url, self.traverse_pathname, id)
			}
		};

		let mut chapters: Vec<Chapter> = Vec::new();
		let html = Request::new(url, HttpMethod::Get)
			.header("User-Agent", USER_AGENT)
			.html()?;
		for chapter in html.select(self.chapter_selector).array() {
			let chapter_node = chapter.as_node().expect("Failed to get chapter as node");
			let raw_title = chapter_node.select(self.chapter_title).text().read();
			let title = {
				// Because every mangastream source likes to be different and not have a
				// consistent chapter naming scheme we have to do some hacky stuff to get the
				// chapter title because we can't use regex

				let mut title = raw_title.split_whitespace().collect::<Vec<&str>>();
				if title.len() >= 2 && title[0] == "Chapter" && title[1].parse::<f64>().is_ok() {
					title.remove(0);
					title.remove(0);
				}

				// Remove any leading hyphens
				if !title.is_empty() && title[0] == "-" {
					title.remove(0);
				}

				title.join(" ")
			};

			let chapter_url: String;
			let chapter_id: String;

			if self.use_chapter_postids {
				let original_url = chapter_node.select(self.chapter_url).attr("href").read();
				let id = chapter_url_to_postid_mapping
					.get(&original_url)
					.ok_or(AidokuError {
						reason: AidokuErrorKind::Unimplemented, // no better error type available
					})?;

				chapter_id = String::from(id);
				chapter_url = format!("{}/?p={}", self.base_url, chapter_id);
			} else {
				chapter_url = {
					let original_url = chapter_node.select(self.chapter_url).attr("href").read();

					if self.has_permanent_chapter_url {
						get_permanet_url(original_url)
					} else {
						original_url
					}
				};

				chapter_id = get_id_from_url(chapter_url.clone());
			}

			let chapter_number = get_chapter_number(raw_title.clone());
			let date_updated = get_date(self, chapter_node.select(self.chapter_date).text());

			chapters.push(Chapter {
				id: chapter_id,
				title,
				chapter: chapter_number,
				date_updated,
				url: chapter_url,
				lang: String::from(self.language),
				..Default::default()
			});
		}
		Ok(chapters)
	}

	//parse the manga chapter images list
	pub fn parse_page_list(&self, id: String) -> Result<Vec<Page>> {
		let url = if self.use_chapter_postids {
			format!("{}/?p={}", self.base_url, id)
		} else if self.has_random_chapter_prefix {
			format!("{}/{}/{}", self.base_url, 0, id)
		} else {
			format!("{}/{}", self.base_url, id)
		};

		let mut pages: Vec<Page> = Vec::new();
		let html = Request::new(url, HttpMethod::Get)
			.header("Referer", &self.base_url)
			.header("User-Agent", USER_AGENT)
			.html()?;
		if self.alt_pages {
			let raw_text = html.select("script").html().read();

			let trimmed_json = {
				let mut trimmed_json = &raw_text[raw_text.find(r#":[{"s"#).unwrap_or(0) + 2
					..raw_text
						.rfind("}]});")
						.unwrap_or(raw_text.rfind("}],").unwrap_or(0))
						+ 1];

				if trimmed_json.contains("Default 2") {
					trimmed_json = &trimmed_json[..trimmed_json.rfind(r#",{"s"#).unwrap_or(0)];
				}

				// if "post_id" is present, we've gone too far
				if trimmed_json.contains("post_id") {
					trimmed_json = &trimmed_json[..trimmed_json.find("}],").unwrap_or(0) + 1];
				}

				trimmed_json
			};

			let json = parse(trimmed_json.as_bytes())?.as_object()?;
			let images = json.get("images").as_array()?;
			for (index, page) in images.enumerate() {
				let page_url = urlencode(page.as_string()?.read());
				pages.push(Page {
					index: index as i32,
					url: page_url,
					..Default::default()
				});
			}
			Ok(pages)
		} else {
			for (at, page) in html.select(self.page_selector).array().enumerate() {
				let page_node = page.as_node().expect("Failed to get page as node");
				let page_url = if self.protocol {
					format!("https:{}", urlencode(page_node.attr(self.page_url).read()))
				} else {
					urlencode(page_node.attr(self.page_url).read())
				};
				// avoid svgs
				if page_url.starts_with("data") {
					continue;
				}
				pages.push(Page {
					index: at as i32,
					url: page_url,
					..Default::default()
				});
			}
			Ok(pages)
		}
	}

	pub fn modify_image_request(&self, request: Request) {
		request
			.header(
				"Accept",
				"image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
			)
			.header("Referer", &self.base_url)
			.header("User-Agent", USER_AGENT);
	}

	pub fn handle_url(&self, url: String) -> Result<DeepLink> {
		let id = get_id_from_url(url);
		Ok(DeepLink {
			manga: Self::parse_manga_details(self, id).ok(),
			chapter: None,
		})
	}
}
