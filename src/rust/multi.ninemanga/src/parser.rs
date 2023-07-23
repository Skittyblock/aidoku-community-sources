use aidoku::{
	error::Result, prelude::format, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaViewer,
	Page,
};

use crate::helper::*;

pub struct NineMangaSource {
	pub base_url: String,
	pub language: String,
	pub date_format: &'static str,
	pub completed_series: &'static str,
	pub date_locale: &'static str,
}

impl Default for NineMangaSource {
	fn default() -> Self {
		NineMangaSource {
			base_url: String::from("https://www.ninemanga.com"),
			language: String::from("en"),
			date_format: "MMM d, yyyy",
			completed_series: "Completed Series",
			date_locale: "en_US",
		}
	}
}

impl NineMangaSource {
	// parse the homepage and filters
	pub fn parse_manga_list(&self, filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
		let mut included_tags: Vec<String> = Vec::new();
		let mut excluded_tags: Vec<String> = Vec::new();
		let mut title: String = String::new();
		let mut status: String = String::new();

		for filter in filters {
			match filter.kind {
				FilterType::Title => {
					title = filter.value.as_string()?.read();
				}

				FilterType::Genre => match filter.value.as_int().unwrap_or(-1) {
					0 => excluded_tags.push(filter.object.get("id").as_string()?.read()),
					1 => included_tags.push(filter.object.get("id").as_string()?.read()),
					_ => continue,
				},

				FilterType::Select => {
					if filter.name.as_str() == self.completed_series {
						match filter.value.as_int().unwrap_or(-1) {
							1 => status = String::from("yes"),
							2 => status = String::from("no"),
							_ => continue,
						}
					}
				}
				_ => continue,
			};
		}

		let url = get_search_url(
			&self.base_url,
			title,
			included_tags,
			excluded_tags,
			status,
			page,
		);

		Self::parse_manga_listing(self, url, page)
	}

	// parse the listing page (popular, latest , new etc)
	pub fn parse_manga_listing(&self, url: String, _page: i32) -> Result<MangaPageResult> {
		let mut mangas: Vec<Manga> = Vec::new();
		let mut has_more = false;

		let html = Request::new(url.as_str(), HttpMethod::Get)
			.html()
			.expect("Failed to initialize the request");

		for manga in html.select(".direlist dl").array() {
			let manga_node = manga.as_node().expect("Failed to get the node");
			let title = manga_node.select("a.bookname").text().read();
			let id = get_manga_id(&manga_node.select("a.bookname").attr("href").read());
			let cover = manga_node.select("img").attr("src").read();
			mangas.push(Manga {
				id,
				cover,
				title,
				..Default::default()
			});
		}

		if url.contains("search") && html.select(".pagelist .l").text().read().contains(">>") {
			has_more = true
		}

		Ok(MangaPageResult {
			manga: mangas,
			has_more,
		})
	}

	// parse manga details page
	pub fn parse_manga_details(&self, id: String) -> Result<Manga> {
		let url = format!("{}/manga/{}", self.base_url, id);
		let html = Request::new(url.as_str(), HttpMethod::Get)
			.header("Accept-Language", "es-ES,es;q=0.9,en;q=0.8,gl;q=0.7")
			.header(
				"User-Agent",
				"Mozilla/5.0 (Windows NT 10.0; WOW64) Gecko/20100101 Firefox/75",
			)
			.html()
			.expect("Failed to initialize the request");

		let title = String::from(
			html.select(".bookintro li")
				.select("span")
				.text()
				.read()
				.replace("Manga", "")
				.trim(),
		);

		let cover = html.select(".bookface img").attr("src").read();

		let author = html
			.select(".bookintro li")
			.select("[itemprop='author']")
			.text()
			.read();

		let description = String::from(
			html.select(".bookintro p")
				.text()
				.read()
				.replace("Summary:", "")
				.trim(),
		);

		let status = status_from_string(html.select(".bookintro .red").first().text().read());
		let mut categories = Vec::new();
		let mut nsfw = MangaContentRating::Safe;
		let mut viewer = MangaViewer::Rtl;

		let nsfw_genres = [
			"Adult",
			"Mature",
			"Ecchi",
			"Smut",
			"adulto",
			"Maduro",
			"Hentai",
			"Adulto",
			"Adulto (18+)",
			"Adulto (YAOI)",
			"Adulte",
		];

		for node in html.select("[itemprop='genre'] a").array() {
			let node = node.as_node().expect("Failed to get the node");
			let category = node.text().read();

			if nsfw_genres.contains(&category.as_str()) {
				nsfw = MangaContentRating::Nsfw;
			}

			if category.as_str() == "Webtoon" {
				viewer = MangaViewer::Scroll;
			}

			categories.push(category);
		}

		Ok(Manga {
			id: id.clone(),
			cover,
			title,
			author,
			description,
			url: format!("{}/manga/{}", self.base_url, id),
			categories,
			status,
			nsfw,
			viewer,
			..Default::default()
		})
	}

	// parse the chapters list present on manga details page
	pub fn parse_chapter_list(&self, id: String) -> Result<Vec<Chapter>> {
		let mut chapters: Vec<Chapter> = Vec::new();

		let url = format!("{}/manga/{}?waring=1", self.base_url, id);
		let html = Request::new(url.as_str(), HttpMethod::Get)
			.header("Accept-Language", "es-ES,es;q=0.9,en;q=0.8,gl;q=0.7")
			.header(
				"User-Agent",
				"Mozilla/5.0 (Windows NT 10.0; WOW64) Gecko/20100101 Firefox/75",
			)
			.html()
			.expect("Failed to initialize the request");

		let name = String::from(
			html.select(".bookface img")
				.attr("alt")
				.read()
				.replace("Manga", "")
				.replace("Манга", "")
				.trim(),
		);

		for chapter in html.select("ul.sub_vol_ul > li").array() {
			let chapter_node = chapter.as_node().expect("Failed to get the node");

			let raw_title = String::from(
				chapter_node
					.select("a")
					.text()
					.read()
					.replace("13610", "")
					.trim(),
			);

			let url = chapter_node.select("a.chapter_list_a").attr("href").read();
			let chapter_id = get_manga_id(&url);
			let chapter_number = extract_f32_from_string(&raw_title, &name);
			let date_updated = get_date(chapter_node, self.date_format, self.date_locale);

			chapters.push(Chapter {
				id: chapter_id,
				chapter: chapter_number,
				date_updated,
				url,
				lang: self.language.clone(),
				..Default::default()
			});
		}

		Ok(chapters)
	}

	//parse the maga chapter images list
	pub fn parse_page_list(&self, id: String, chapter_id: String) -> Result<Vec<Page>> {
		let mut pages: Vec<Page> = Vec::new();
		let mut at = 0;

		let pages_arr = get_chapter_pages(
			&self.base_url,
			format!("{}/chapter/{}/{}", self.base_url, id, chapter_id).as_str(),
		);
		for url in pages_arr {
			let html = Request::new(&url, HttpMethod::Get)
				.header("Accept-Language", "es-ES,es;q=0.9,en;q=0.8,gl;q=0.7")
				.header(
					"User-Agent",
					"Mozilla/5.0 (Windows NT 10.0; WOW64) Gecko/20100101 Firefox/75",
				)
				.header("Cookie", "ninemanga_webp_valid=true")
				.html()
				.expect("Failed to initialize the request");

			for page in html.select(".pic_box img.manga_pic").array() {
				let page_node = page.as_node().expect("Failed to get the node");
				let page_url = page_node.attr("src").read();
				pages.push(Page {
					index: at,
					url: page_url,
					..Default::default()
				});
				at += 1;
			}
		}

		Ok(pages)
	}

	pub fn modify_image_request(&self, request: Request) {
		request
			.header("Referer", self.base_url.as_str())
			.header("Accept-Language", "es-ES,es;q=0.9,en;q=0.8,gl;q=0.7")
			.header(
				"User-Agent",
				"Mozilla/5.0 (Windows NT 10.0; WOW64) Gecko/20100101 Firefox/75",
			);
	}

	pub fn handle_url(&self, url: String) -> Result<DeepLink> {
		let id = get_manga_id(&url);
		Ok(DeepLink {
			manga: Some(Self::parse_manga_details(self, id)?),
			chapter: None,
		})
	}
}
