use crate::{helper::*, html_entity_decoder::decode_html_entities};
use aidoku::{
	error::Result,
	prelude::*,
	std::{
		html::Node,
		net::{HttpMethod, Request},
		ArrayRef, String, StringRef, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

pub struct MyMangaSource {
	pub base_url: &'static str,
	pub language: &'static str,
	pub manga_details_path: &'static str,

	pub date_format: &'static str,
	pub timezone: &'static str,
	pub should_split_to_get_date: bool,
	pub split_str: &'static str,
}

impl Default for MyMangaSource {
	fn default() -> Self {
		MyMangaSource {
			base_url: "https://teamojisan.com",
			language: "vi",
			manga_details_path: "/truyen",
			date_format: "dd/MM/yyyy",
			timezone: "Asia/Ho_Chi_Minh",
			should_split_to_get_date: false,
			split_str: " - ",
		}
	}
}

static mut CACHED_MANGA_ID: Option<String> = None;
static mut CACHED_MANGA: Option<Vec<u8>> = None;

fn cache_manga_page(url: &str) {
	if unsafe { CACHED_MANGA_ID.is_some() } && unsafe { CACHED_MANGA_ID.clone().unwrap() } == url {
		return;
	}

	unsafe {
		CACHED_MANGA = Some(Request::new(url, HttpMethod::Get).data());
		CACHED_MANGA_ID = Some(String::from(url));
	};
}

impl MyMangaSource {
	fn parse_manga_list(&self, elems: ArrayRef) -> (Vec<Manga>, bool) {
		let mut manga: Vec<Manga> = Vec::with_capacity(elems.len());
		let has_more = elems.len() > 0;
		for elem in elems {
			let node = elem.as_node();
			let url = node.select("a").attr("href").read();
			let id = String::from(&url[self.base_url.len()..]);
			let cover = node
				.select("div[data-bg]")
				.attr("data-bg")
				.read()
				.replace("http:", "https:");
			let title = String::from(
				node.select("div.thumb_attr.series-title a[title]")
					.text()
					.read()
					.trim(),
			);
			manga.push(Manga {
				id,
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
		(manga, has_more)
	}

	pub fn get_manga_list(&self, filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
		let mut included_tags: Vec<String> = Vec::new();
		let mut excluded_tags: Vec<String> = Vec::new();
		let mut url = format!("{}/tim-kiem?page={page}", self.base_url);

		for filter in filters {
			match filter.kind {
				FilterType::Title => {
					let title = urlencode(filter.value.as_string()?.read());
					url.push_str("&q=");
					url.push_str(title.as_str());
				}
				FilterType::Author => {
					let author = urlencode(filter.value.as_string()?.read());
					url.push_str("&artist=");
					url.push_str(author.as_str());
				}
				FilterType::Genre => {
					let id = filter.object.get("id").as_string()?.read();
					match filter.value.as_int().unwrap_or(-1) {
						0 => excluded_tags.push(id),
						1 => included_tags.push(id),
						_ => continue,
					}
				}
				_ => match filter.name.as_str() {
					"Sắp xếp" => {
						url.push_str("&sort=");
						match filter.value.as_int().unwrap_or(-1) {
							0 => url.push_str("az"),
							1 => url.push_str("za"),
							2 => url.push_str("update"),
							3 => url.push_str("new"),
							4 => url.push_str("top"),
							5 => url.push_str("like"),
							_ => continue,
						}
					}
					"Tình trạng" => {
						let value = filter.value.as_int().unwrap_or(-1);
						if value <= 0 {
							continue;
						}
						url.push_str(format!("&status={}", value).as_str());
					}
					_ => continue,
				},
			}
		}
		if !excluded_tags.is_empty() {
			url.push_str(format!("&reject_genres={}", excluded_tags.join(",")).as_str());
		}
		if !included_tags.is_empty() {
			url.push_str(format!("&accept_genres={}", included_tags.join(",")).as_str());
		}
		let html = Request::new(&url, HttpMethod::Get).html();
		let node = html.select("div.thumb-item-flow.col-6.col-md-2");
		let elems = node.array();
		let (manga, has_more) = self.parse_manga_list(elems);
		Ok(MangaPageResult { manga, has_more })
	}

	pub fn get_manga_details(&self, id: String) -> Result<Manga> {
		let url = format!("{}{id}", self.base_url);
		cache_manga_page(&url);
		let html = unsafe { Node::new(&CACHED_MANGA.clone().unwrap()) };
		let title = String::from(html.select("span.series-name").text().read().trim());
		let author = String::from(
			html.select("div.info-item:contains(Tác giả) span.info-value")
				.text()
				.read()
				.trim(),
		);
		let status = match html
			.select("div.info-item:contains(Tình trạng) span.info-value")
			.text()
			.read()
			.trim()
		{
			"Đang tiến hành" => MangaStatus::Ongoing,
			"Tạm ngưng" => MangaStatus::Hiatus,
			"Đã hoàn thành" => MangaStatus::Completed,
			_ => MangaStatus::Unknown,
		};
		let cover_style = html.select("div.content.img-in-ratio").attr("style").read();
		let cover = cover_style.split('(').collect::<Vec<_>>()[1]
			.split(')')
			.next()
			.unwrap_or_default()
			.replace(|char| char == '"' || char == '\'', "");
		let description = text_with_newlines(html.select("div.summary-content"));
		let categories = html
			.select("a[href*=the-loai] span.badge")
			.array()
			.map(|elem| {
				let node = elem.as_node();
				String::from(node.text().read().trim())
			})
			.collect::<Vec<_>>();
		let (nsfw, viewer) = category_parser(&categories);
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

	pub fn get_chapter_list(&self, id: String) -> Result<Vec<Chapter>> {
		let url = format!("{}{id}", self.base_url);
		cache_manga_page(&url);
		let html = unsafe { Node::new(&CACHED_MANGA.clone().unwrap()) };
		let scanlator = {
			let original = String::from(html.select("div.fantrans-value a").text().read().trim());
			let temp = decode_html_entities(&original);
			if temp == "Đang cập nhật" {
				String::new()
			} else {
				String::from(temp)
			}
		};
		let node = html.select("ul.list-chapters > a");
		let elems = node.array();
		let mut chapters = Vec::with_capacity(elems.len());
		for elem in elems {
			let chapter_node = elem.as_node();
			let url = chapter_node.attr("href").read();
			let id = String::from(&url[self.base_url.len()..]);
			let date_updated = if self.should_split_to_get_date {
				let original = chapter_node.select("div.chapter-time").text().read();
				StringRef::from(original.split(self.split_str).collect::<Vec<_>>()[1])
					.0
					.as_date(self.date_format, None, Some(self.timezone))
			} else {
				chapter_node.select("div.chapter-time").text().0.as_date(
					self.date_format,
					None,
					Some(self.timezone),
				)
			}
			.unwrap_or(-1.0);
			let mut title = String::from(decode_html_entities(
				&chapter_node.select("div.chapter-name").text().read(),
			));
			let numbers = extract_f32_from_string(String::new(), String::from(&title));
			let (volume, chapter) =
				if numbers.len() > 1 && title.to_ascii_lowercase().contains("vol") {
					(numbers[0], numbers[1])
				} else if !numbers.is_empty() {
					(-1.0, numbers[0])
				} else {
					(-1.0, -1.0)
				};
			if chapter >= 0.0 {
				let splitter = format!(" {}", chapter);
				if title.contains(&splitter) {
					let split = title.splitn(2, &splitter).collect::<Vec<&str>>();
					title =
						String::from(split[1]).replacen(|char| char == ':' || char == '-', "", 1);
				}
			}
			chapters.push(Chapter {
				id,
				title: String::from(title.trim()),
				volume,
				chapter,
				date_updated,
				scanlator: scanlator.clone(),
				url,
				lang: String::from(self.language),
			})
		}
		Ok(chapters)
	}

	pub fn get_page_list(&self, id: String) -> Result<Vec<Page>> {
		let url = format!("{}{id}", self.base_url);
		let html = Request::new(&url, HttpMethod::Get).html();
		let node = html.select("div#chapter-content img");
		let elems = node.array();
		let mut pages = Vec::with_capacity(elems.len());
		for (idx, elem) in elems.enumerate() {
			let node = elem.as_node();
			let url = node.attr("data-src").read();
			pages.push(Page {
				index: idx as i32,
				url,
				base64: String::new(),
				text: String::new(),
			})
		}
		Ok(pages)
	}

	pub fn modify_image_request(&self, request: Request) {
		let mut referer_url = String::from(self.base_url);
		referer_url.push('/');
		request.header("Referer", &referer_url);
	}

	pub fn handle_url(&self, url: String) -> Result<DeepLink> {
		let id = String::from(&url[self.base_url.len()..]);
		let id_split = id.split('/').collect::<Vec<_>>();
		if id_split.len() == 4 {
			let manga_id = format!("/{}", id_split[1..=2].join("/"));
			Ok(DeepLink {
				manga: Some(self.get_manga_details(manga_id)?),
				chapter: Some(Chapter {
					id,
					title: String::new(),
					volume: -1.0,
					chapter: -1.0,
					date_updated: -1.0,
					scanlator: String::new(),
					url,
					lang: String::from("vi"),
				}),
			})
		} else if id.contains(self.manga_details_path) {
			Ok(DeepLink {
				manga: Some(self.get_manga_details(id)?),
				chapter: None,
			})
		} else {
			unreachable!()
		}
	}
}
