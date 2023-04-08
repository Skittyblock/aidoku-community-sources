use aidoku::{
	error::Result, prelude::*, std::html::Node, std::net::HttpMethod, std::net::Request,
	std::String, std::StringRef, std::Vec, Chapter, DeepLink, Listing, Manga, MangaContentRating,
	MangaPageResult, MangaStatus, MangaViewer, Page,
};

use crate::helper::{append_protocol, extract_f32_from_string, text_with_newlines};

pub struct WPComicsSource {
	pub base_url: String,
	pub listing_mapping: fn(String) -> String,
	pub status_mapping: fn(String) -> MangaStatus,
	pub time_converter: fn(String) -> f64,
	pub nsfw: MangaContentRating,
	pub viewer: MangaViewer,

	pub next_page: &'static str,
	pub manga_cell: &'static str,
	pub manga_cell_url: &'static str,
	pub manga_cell_title: &'static str,
	pub manga_cell_image: &'static str,
	pub manga_cell_image_attr: &'static str,

	// https://{base_url}/{listing}{manga_listing_pagination}{page}{manga_listing_extension}
	pub manga_listing_pagination: &'static str,
	pub manga_listing_extension: &'static str,

	pub manga_details_title: &'static str,
	pub manga_details_title_transformer: fn(String) -> String,
	pub manga_details_cover: &'static str,
	pub manga_details_author: &'static str,
	pub manga_details_author_transformer: fn(String) -> String,
	pub manga_details_description: &'static str,
	pub manga_details_tags: &'static str,
	pub manga_details_tags_splitter: &'static str,
	pub manga_details_status: &'static str,
	pub manga_details_status_transformer: fn(String) -> String,

	pub manga_details_chapters: &'static str,
	pub chapter_skip_first: bool,
	pub chapter_date_selector: &'static str,
	pub chapter_anchor_selector: &'static str,

	pub manga_viewer_page: &'static str,
	pub manga_viewer_page_url_suffix: &'static str,
	pub page_url_transformer: fn(String) -> String,

	pub vinahost_protection: bool,
}

static mut CACHED_MANGA_ID: Option<String> = None;
static mut CACHED_MANGA: Option<Vec<u8>> = None;
static mut VINAHOST_COOKIE: Option<String> = None;

fn cache_manga_page(data: &WPComicsSource, url: &str) {
	if unsafe { CACHED_MANGA_ID.is_some() } && unsafe { CACHED_MANGA_ID.clone().unwrap() } == url {
		return;
	}

	unsafe {
		if data.vinahost_protection {
			CACHED_MANGA = Some(data.request_vinahost(url).data());
		} else {
			CACHED_MANGA = Some(Request::new(url, HttpMethod::Get).data());
		}
		CACHED_MANGA_ID = Some(String::from(url));
	};
}

impl WPComicsSource {
	fn request_vinahost(&self, url: &str) -> Request {
		if unsafe { VINAHOST_COOKIE.is_some() } {
			Request::new(url, HttpMethod::Get).header(
				"Cookie",
				unsafe { VINAHOST_COOKIE.clone().unwrap() }.as_str(),
			)
		} else if self.vinahost_protection {
			if let Ok(blocked_html) = Request::new(url, HttpMethod::Get).html() {
				let script = blocked_html.select("script").html().read();
				let cookie = script
					.replace("document.cookie=\"", "")
					.replace("\";window.location.reload(true);", "")
					.replace("\"+\"", "");
				unsafe {
					VINAHOST_COOKIE = Some(cookie);
				};
				Request::new(url, HttpMethod::Get).header(
					"Cookie",
					unsafe { VINAHOST_COOKIE.clone().unwrap() }.as_str(),
				)
			} else {
				Request::new(url, HttpMethod::Get)
			}
		} else {
			Request::new(url, HttpMethod::Get)
		}
	}

	fn category_parser(&self, categories: &Vec<String>) -> (MangaContentRating, MangaViewer) {
		#[allow(clippy::needless_match)]
		let mut nsfw = match self.nsfw {
			MangaContentRating::Safe => MangaContentRating::Safe,
			MangaContentRating::Suggestive => MangaContentRating::Suggestive,
			MangaContentRating::Nsfw => MangaContentRating::Nsfw,
		};
		#[allow(clippy::needless_match)]
		let mut viewer = match self.viewer {
			MangaViewer::Rtl => MangaViewer::Rtl,
			MangaViewer::Ltr => MangaViewer::Ltr,
			MangaViewer::Scroll => MangaViewer::Scroll,
			MangaViewer::Vertical => MangaViewer::Vertical,
			_ => MangaViewer::Rtl,
		};
		for category in categories {
			match category.as_str() {
				"Smut" | "Mature" | "18+" => nsfw = MangaContentRating::Nsfw,
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
	}

	pub fn get_manga_list(&self, search_url: String) -> Result<MangaPageResult> {
		let mut has_next_page = !self.next_page.is_empty();
		let html = self.request_vinahost(&search_url).html()?;
		let node = html.select(self.manga_cell);
		let elems = node.array();
		let mut mangas: Vec<Manga> = Vec::with_capacity(elems.len());
		for item in elems {
			let item_node = item.as_node().expect("node array");
			let title = item_node
				.select(self.manga_cell_title)
				.first()
				.text()
				.read();
			let mut id = item_node
				.select(self.manga_cell_url)
				.first()
				.attr("href")
				.read();
			if !id.contains("http://") && !id.contains("https://") {
				id = String::from(&self.base_url) + "/" + &id;
			}
			let cover = if !self.manga_cell_image.is_empty() {
				append_protocol(
					item_node
						.select(self.manga_cell_image)
						.first()
						.attr(self.manga_cell_image_attr)
						.read(),
				)
			} else {
				String::new()
			};
			mangas.push(Manga {
				id,
				cover,
				title: (self.manga_details_title_transformer)(title),
				..Default::default()
			});
		}
		if !self.next_page.is_empty() {
			has_next_page = html.select(self.next_page).array().is_empty();
		}
		Ok(MangaPageResult {
			manga: mangas,
			has_more: has_next_page,
		})
	}

	pub fn get_manga_listing(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		let url = format!(
			"{}/{}{}{page}{}",
			self.base_url,
			(self.listing_mapping)(listing.name),
			self.manga_listing_pagination,
			self.manga_listing_extension
		);
		self.get_manga_list(url)
	}

	pub fn get_manga_details(&self, id: String) -> Result<Manga> {
		cache_manga_page(self, id.as_str());
		let details = unsafe { Node::new(&CACHED_MANGA.clone().unwrap())? };
		let title = details.select(self.manga_details_title).text().read();
		let cover = append_protocol(details.select(self.manga_details_cover).attr("src").read());
		let author = (self.manga_details_author_transformer)(
			details.select(self.manga_details_author).text().read(),
		);
		let description = text_with_newlines(details.select(self.manga_details_description));
		let mut categories = Vec::new();

		if !self.manga_details_tags.is_empty() {
			if self.manga_details_tags_splitter.is_empty() {
				categories = details
					.select(self.manga_details_tags)
					.array()
					.map(|elem| elem.as_node().expect("node array").text().read())
					.collect::<Vec<_>>();
			} else {
				for node in details
					.select(self.manga_details_tags)
					.text()
					.read()
					.split(self.manga_details_tags_splitter)
				{
					categories.push(String::from(node));
				}
			}
		}
		let (nsfw, viewer) = self.category_parser(&categories);
		let status = (self.status_mapping)((self.manga_details_status_transformer)(
			details.select(self.manga_details_status).text().read(),
		));
		Ok(Manga {
			id: id.clone(),
			cover,
			title: (self.manga_details_title_transformer)(title),
			author,
			artist: String::new(),
			description,
			url: id,
			categories,
			status,
			nsfw,
			viewer,
		})
	}

	pub fn get_chapter_list(&self, id: String) -> Result<Vec<Chapter>> {
		let mut skipped_first = false;
		let mut chapters: Vec<Chapter> = Vec::new();
		cache_manga_page(self, id.as_str());
		let html = unsafe { Node::new(&CACHED_MANGA.clone().unwrap())? };
		let title_untrimmed = (self.manga_details_title_transformer)(
			html.select(self.manga_details_title).text().read(),
		);
		let title = title_untrimmed.trim();
		for chapter in html.select(self.manga_details_chapters).array() {
			if self.chapter_skip_first && !skipped_first {
				skipped_first = true;
				continue;
			}
			let chapter_node = chapter.as_node().expect("node array");
			let mut chapter_url = chapter_node
				.select(self.chapter_anchor_selector)
				.attr("href")
				.read();
			if !chapter_url.contains("http://") && !chapter_url.contains("https://") {
				chapter_url = format!("{}/{}", self.base_url, chapter_url);
			}
			let chapter_id = chapter_url.clone();
			let mut chapter_title = chapter_node
				.select(self.chapter_anchor_selector)
				.text()
				.read();
			let numbers =
				extract_f32_from_string(String::from(title), String::from(&chapter_title));
			let (volume, chapter) =
				if numbers.len() > 1 && chapter_title.to_ascii_lowercase().contains("vol") {
					(numbers[0], numbers[1])
				} else if !numbers.is_empty() {
					(-1.0, numbers[0])
				} else {
					(-1.0, -1.0)
				};
			if chapter >= 0.0 {
				let splitter = format!(" {}", chapter);
				let splitter2 = format!("#{}", chapter);
				if chapter_title.contains(&splitter) {
					let split = chapter_title.splitn(2, &splitter).collect::<Vec<&str>>();
					chapter_title =
						String::from(split[1]).replacen(|char| char == ':' || char == '-', "", 1);
				} else if chapter_title.contains(&splitter2) {
					let split = chapter_title.splitn(2, &splitter2).collect::<Vec<&str>>();
					chapter_title =
						String::from(split[1]).replacen(|char| char == ':' || char == '-', "", 1);
				}
			}
			let date_updated = (self.time_converter)(
				chapter_node
					.select(self.chapter_date_selector)
					.text()
					.read(),
			);
			chapters.push(Chapter {
				id: chapter_id,
				title: String::from(chapter_title.trim()),
				volume,
				chapter,
				date_updated,
				scanlator: String::new(),
				url: chapter_url,
				lang: String::from("en"),
			});
		}
		Ok(chapters)
	}

	pub fn get_page_list(&self, chapter_id: String) -> Result<Vec<Page>> {
		let mut pages: Vec<Page> = Vec::new();
		let url = format!("{}{}", &chapter_id, self.manga_viewer_page_url_suffix);
		let html = self.request_vinahost(&url).html()?;
		for (at, page) in html.select(self.manga_viewer_page).array().enumerate() {
			let page_node = page.as_node().expect("node array");
			let mut page_url = page_node.attr("data-original").read();
			if !page_url.starts_with("http") {
				page_url = String::from("https:") + &page_url;
			}
			pages.push(Page {
				index: at as i32,
				url: (self.page_url_transformer)(page_url),
				base64: String::new(),
				text: String::new(),
			});
		}
		Ok(pages)
	}

	pub fn handle_url(&self, url: String) -> Result<DeepLink> {
		cache_manga_page(self, url.as_str());
		let html = unsafe { Node::new(CACHED_MANGA.clone().unwrap())? };
		if html.select(self.manga_viewer_page).array().is_empty() {
			let node = html.select(".breadcrumb li");
			let breadcrumbs = node.array();
			let mut manga_id = breadcrumbs
				.get(breadcrumbs.len() / 2 - 2)
				.as_node()
				.expect("node array")
				.select("a")
				.attr("href")
				.read();
			if !manga_id.contains("http://") && !manga_id.contains("https://") {
				manga_id = format!("{}/{}", self.base_url, manga_id);
			}
			Ok(DeepLink {
				manga: Some(self.get_manga_details(manga_id)?),
				chapter: Some(Chapter {
					id: url.clone(),
					title: String::new(),
					volume: -1.0,
					chapter: -1.0,
					date_updated: -1.0,
					scanlator: String::new(),
					url,
					lang: String::new(),
				}),
			})
		} else {
			Ok(DeepLink {
				manga: Some(self.get_manga_details(url)?),
				chapter: None,
			})
		}
	}

	pub fn modify_image_request(&self, request: Request) {
		request.header("Referer", format!("{}/", self.base_url).as_str());
	}
}

impl Default for WPComicsSource {
	fn default() -> WPComicsSource {
		WPComicsSource {
			base_url: String::new(),
			listing_mapping: |str| str,
			status_mapping: |status| match status.as_str() {
				"Ongoing" => MangaStatus::Ongoing,
				"Completed" => MangaStatus::Completed,
				"Hiatus" => MangaStatus::Hiatus,
				"Cancelled" => MangaStatus::Cancelled,
				_ => MangaStatus::Unknown,
			},
			time_converter: |str| {
				StringRef::from(str)
					.0
					.as_date("MM/dd/yyyy", Some("en_US"), None)
					.unwrap_or(0.0)
			},
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Ltr,

			next_page: "li > a[rel=next]",
			manga_cell: "div.items > div.row > div.item > figure.clearfix",
			manga_cell_title: "figcaption > h3 > a",
			manga_cell_url: "figcaption > h3 > a",
			manga_cell_image: "div.image > a > img",
			manga_cell_image_attr: "data-original",

			manga_listing_pagination: "?page=",
			manga_listing_extension: "",

			manga_details_title: "h1.title-detail",
			manga_details_title_transformer: |title| title,
			manga_details_cover: "div.col-image > img",
			manga_details_author: "ul.list-info > li.author > p.col-xs-8",
			manga_details_author_transformer: |title| title,
			manga_details_description: "div.detail-content > p",
			manga_details_tags: "li.kind.row > p.col-xs-8",
			manga_details_tags_splitter: " - ",
			manga_details_status: "li.status.row > p.col-xs-8",
			manga_details_status_transformer: |title| title,
			manga_details_chapters: "div.list-chapter > nav > ul > li",

			chapter_skip_first: false,
			chapter_anchor_selector: "div.chapter > a",
			chapter_date_selector: "div.col-xs-4",

			manga_viewer_page: "div.page-chapter > img",
			manga_viewer_page_url_suffix: "",
			page_url_transformer: |url| url,

			vinahost_protection: false,
		}
	}
}

pub fn modify_image_request(base_url: String, user_agent: String, request: Request) {
	request
		.header("Referer", &base_url)
		.header("User-Agent", &user_agent);
}
