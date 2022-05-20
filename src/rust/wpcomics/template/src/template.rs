use aidoku::{
	error::Result, prelude::format, std::net::HttpMethod, std::net::Request, std::String,
	std::StringRef, std::Vec, Chapter, DeepLink, Listing, Manga, MangaContentRating,
	MangaPageResult, MangaStatus, MangaViewer, Page,
};

use crate::helper::{append_protocol, extract_f32_from_string, https_upgrade};

pub struct WPComicsSource {
	pub base_url: &'static str,
	pub listing_mapping: fn(String) -> String,
	pub status_mapping: fn(String) -> MangaStatus,
	pub time_converter: fn(String) -> f64,

	pub next_page: &'static str,
	pub manga_cell: &'static str,
	pub manga_cell_url: &'static str,
	pub manga_cell_title: &'static str,
	pub manga_cell_image: &'static str,

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
}

impl WPComicsSource {
	pub fn get_manga_list(&self, search_url: String) -> Result<MangaPageResult> {
		let mut mangas: Vec<Manga> = Vec::new();
		let mut has_next_page = self.next_page.len() > 0;
		let html = Request::new(&search_url, HttpMethod::Get).html();
		for item in html.select(self.manga_cell).array() {
			let item_node = item.as_node();
			let title = item_node
				.select(self.manga_cell_title)
				.first()
				.text()
				.read();
			let id = https_upgrade(
				item_node
					.select(self.manga_cell_url)
					.first()
					.attr("href")
					.read(),
			);
			let cover = if self.manga_cell_image != "" {
				https_upgrade(append_protocol(
					item_node
						.select(self.manga_cell_image)
						.first()
						.attr("data-original")
						.read(),
				))
			} else {
				String::new()
			};
			mangas.push(Manga {
				id,
				cover,
				title: (self.manga_details_title_transformer)(title),
				author: String::new(),
				artist: String::new(),
				description: String::new(),
				url: String::new(),
				categories: Vec::new(),
				status: MangaStatus::Unknown,
				nsfw: MangaContentRating::Safe,
				viewer: MangaViewer::Default,
			});
		}
		if self.next_page.len() > 0 {
			has_next_page = html.select(self.next_page).array().len() > 0;
		}
		Ok(MangaPageResult {
			manga: mangas,
			has_more: has_next_page,
		})
	}

	pub fn get_manga_listing(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		let url = format!(
			"{}/{}?page={page}",
			self.base_url,
			(self.listing_mapping)(listing.name)
		);
		self.get_manga_list(url)
	}

	pub fn get_manga_details(&self, id: String, default_viewer: MangaViewer) -> Result<Manga> {
		let details = Request::new(id.clone().as_str(), HttpMethod::Get).html();
		let title = details.select(self.manga_details_title).text().read();
		let cover = append_protocol(details.select(self.manga_details_cover).attr("src").read());
		let author = (self.manga_details_author_transformer)(
			details.select(self.manga_details_author).text().read(),
		);
		let description = details.select(self.manga_details_description).text().read();
		let mut categories = Vec::new();
		let mut nsfw = MangaContentRating::Safe;
		let mut viewer = default_viewer;

		if self.manga_details_tags.len() > 0 {
			for node in details
				.select(self.manga_details_tags)
				.text()
				.read()
				.split(self.manga_details_tags_splitter)
			{
				let category = String::from(node);
				if category == String::from("Smut")
					|| category == String::from("Mature")
					|| category == String::from("Adult")
					|| category == String::from("18+")
				{
					nsfw = MangaContentRating::Nsfw;
				} else if category == String::from("Ecchi") || category == String::from("16+") {
					nsfw = MangaContentRating::Suggestive;
				}
				if category.contains("Webtoon")
					|| category.contains("Manhwa")
					|| category.contains("Manhua")
				{
					viewer = MangaViewer::Scroll;
				}
				categories.push(category.clone());
			}
		}
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
			url: id.clone(),
			categories,
			status,
			nsfw,
			viewer,
		})
	}

	pub fn get_chapter_list(&self, id: String) -> Result<Vec<Chapter>> {
		let mut skipped_first = false;
		let mut chapters: Vec<Chapter> = Vec::new();
		let html = Request::new(id.clone().as_str(), HttpMethod::Get).html();
		let title_untrimmed = (self.manga_details_title_transformer)(
			html.select(self.manga_details_title).text().read(),
		);
		let title = title_untrimmed.trim();
		for chapter in html.select(self.manga_details_chapters).array() {
			if self.chapter_skip_first && !skipped_first {
				skipped_first = true;
				continue;
			}
			let chapter_node = chapter.as_node();
			let chapter_url = https_upgrade(
				chapter_node
					.select(self.chapter_anchor_selector)
					.attr("href")
					.read(),
			);
			let chapter_id = chapter_url.clone();
			let chapter_title = chapter_node
				.select(self.chapter_anchor_selector)
				.text()
				.read();
			let chapter_number =
				extract_f32_from_string(String::from(title), String::from(&chapter_title));
			let date_updated = (self.time_converter)(
				chapter_node
					.select(self.chapter_date_selector)
					.text()
					.read(),
			);
			chapters.push(Chapter {
				id: chapter_id,
				title: chapter_title,
				volume: -1.0,
				chapter: chapter_number,
				date_updated,
				scanlator: String::new(),
				url: chapter_url,
				lang: String::from("en"),
			});
		}
		Ok(chapters)
	}

	pub fn get_page_list(&self, id: String) -> Result<Vec<Page>> {
		let mut pages: Vec<Page> = Vec::new();
		let url = format!("{}{}", &id, self.manga_viewer_page_url_suffix);
		let html = Request::new(&url, HttpMethod::Get).html();
		let mut at = 0;
		for page in html.select(self.manga_viewer_page).array() {
			let page_node = page.as_node();
			let mut page_url = page_node.attr("data-original").read();
			if !page_url.starts_with("http") {
				page_url = String::from(String::from("https:") + &page_url);
			}
			pages.push(Page {
				index: at,
				url: (self.page_url_transformer)(page_url),
				base64: String::new(),
				text: String::new(),
			});
			at += 1;
		}
		Ok(pages)
	}

	pub fn handle_url(&self, url: String, default_viewer: MangaViewer) -> Result<DeepLink> {
		Ok(DeepLink {
			manga: Some(self.get_manga_details(url.clone(), default_viewer)?),
			chapter: None,
		})
	}
}

impl Default for WPComicsSource {
	fn default() -> WPComicsSource {
		WPComicsSource {
			base_url: "",
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

			next_page: "li > a[rel=next]",
			manga_cell: "div.items > div.row > div.item > figure.clearfix",
			manga_cell_title: "figcaption > h3 > a",
			manga_cell_url: "figcaption > h3 > a",
			manga_cell_image: "div.image > a > img",

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
		}
	}
}

pub fn modify_image_request(base_url: String, user_agent: String, request: Request) {
	request
		.header("Referer", &base_url)
		.header("User-Agent", &user_agent);
}
