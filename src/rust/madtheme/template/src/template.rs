use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	helpers::{node::NodeHelpers, substring::Substring, uri::QueryParameters},
	prelude::*,
	std::{current_date, html::Node, net::Request, String, StringRef, Vec},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult,
	MangaStatus, Page,
};
use alloc::{string::ToString, vec};

use crate::helper::extract_f32_from_string;

pub trait MadTheme {
	fn base_url(&self) -> &'static str;

	fn status_vec(&self) -> Vec<&'static str> {
		vec!["all", "ongoing", "completed"]
	}

	fn order_vec(&self) -> Vec<&'static str> {
		vec!["views", "updated_at", "created_at", "name", "rating"]
	}

	fn get_manga_list(&self, filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
		let mut qs = QueryParameters::new();
		qs.push("page", Some(&page.to_string()));

		for filter in filters {
			match filter.kind {
				FilterType::Title => {
					let query = filter.value.as_string().unwrap_or_default().read();
					qs.push("q", Some(&query));
				}
				FilterType::Select => {
					let idx = filter.value.as_int();
					match filter.name.as_ref() {
						"Status" => {
							qs.push("status", Some(self.status_vec()[idx.unwrap_or(0) as usize]))
						}
						"Order by" => {
							qs.push("sort", Some(self.order_vec()[idx.unwrap_or(1) as usize]))
						}
						_ => continue,
					}
				}
				FilterType::Genre => {
					if filter.value.as_int().unwrap_or(-1) == 1
					   && let Ok(id) = filter.object.get("id").as_string() {
						qs.push("genre[]", Some(&id.read()))
					}
				}
				_ => continue,
			}
		}
		let url = format!("{}/search?{qs}", self.base_url());
		let req = Request::get(url).header("Referer", self.base_url());
		let document = req.html()?;

		self.parse_manga_list(document)
	}

	fn parse_manga_list(&self, document: Node) -> Result<MangaPageResult> {
		let elems = document.select(".book-detailed-item");
		let nodes = elems.array();
		let mut manga: Vec<Manga> = Vec::with_capacity(nodes.len());

		for node in nodes {
			if let Ok(node) = node.as_node() {
				let id = node.select("a").attr("href").read().replace('/', "");
				let title = node.select("a").attr("title").read();
				let description = node.select(".summary").text_with_newlines();
				let category_nodes = node.select(".genres span");
				let categories = category_nodes
					.array()
					.filter_map(|v| {
						if let Ok(v) = v.as_node() {
							Some(v.text().read())
						} else {
							None
						}
					})
					.collect::<Vec<_>>();
				let cover = node.select("img").attr("abs:data-src").read();
				manga.push(Manga {
					id,
					title,
					description,
					categories,
					cover,
					..Default::default()
				})
			}
		}

		let next_page_elem = document.select(".paginator > a.active + a:not([rel=next])");
		Ok(MangaPageResult {
			manga,
			has_more: !next_page_elem.array().is_empty(),
		})
	}

	fn get_manga_listing(&self, _: Listing, _: i32) -> Result<MangaPageResult> {
		unimplemented!()
	}

	fn get_manga_details(&self, id: String) -> Result<Manga> {
		let url = format!("{}/{id}", self.base_url());
		let request = Request::get(&url).header("Referer", self.base_url());
		let document = request.html()?;

		let title = document.select(".detail h1").first().text().read();

		let author_nodes = document.select(".detail .meta > p > strong:contains(Authors) ~ a");
		let author = author_nodes
			.array()
			.filter_map(|v| v.as_node().map(|v| v.attr("title").read()).ok())
			.collect::<Vec<_>>()
			.join(", ");

		let cover = document.select("#cover img").attr("abs:data-src").read();

		let category_nodes = document.select(".detail .meta > p > strong:contains(Genres) ~ a");
		let categories = category_nodes
			.array()
			.filter_map(|v| {
				v.as_node()
					.map(|v| {
						v.text()
							.read()
							.trim_end_matches(|c| c == ',' || char::is_whitespace(c))
							.to_string()
					})
					.ok()
			})
			.collect::<Vec<_>>();

		let alt_name_node = document.select(".detail h2");
		let alt_name_str = alt_name_node.text().read();
		let alt_names = alt_name_str
			.split(&[',', ';'])
			.filter_map(|v| {
				if v.trim() != title.trim() {
					Some(v.trim())
				} else {
					None
				}
			})
			.collect::<Vec<_>>();

		let mut description = String::new();
		if !alt_names.is_empty() {
			description.push_str(&format!("Alternate titles: {}\n\n", alt_names.join(",")));
		}
		let description_node = document.select(".summary .content");
		description.push_str(&description_node.text_with_newlines());

		let status_node = document.select(".detail .meta > p > strong:contains(Status) ~ a");
		let status_str = status_node.text().read();
		let status = match status_str.to_lowercase().as_ref() {
			"ongoing" => MangaStatus::Ongoing,
			"completed" => MangaStatus::Completed,
			_ => MangaStatus::Unknown,
		};
		let nsfw = if categories.contains(&"Adult".to_string()) {
			MangaContentRating::Nsfw
		} else if categories.contains(&"Ecchi".to_string()) {
			MangaContentRating::Suggestive
		} else {
			MangaContentRating::Safe
		};

		Ok(Manga {
			id,
			title,
			author,
			cover,
			categories,
			description,
			status,
			url,
			nsfw,
			..Default::default()
		})
	}

	fn parse_chapter_date(&self, date: StringRef) -> f64 {
		let date_str = date.read();

		if date_str.ends_with("ago") {
			let curr = current_date();
			let split = date_str.split(char::is_whitespace).collect::<Vec<_>>();
			let denomination = split[1];
			if let Ok(num) = split[0].parse::<f64>() {
				if denomination.contains("day") {
					curr - num * 24.0 * 60.0 * 60.0
				} else if denomination.contains("hour") {
					curr - num * 60.0 * 60.0
				} else if denomination.contains("minute") {
					curr - num * 60.0
				} else if denomination.contains("second") {
					curr - num
				} else {
					-1.0
				}
			} else {
				-1.0
			}
		} else {
			date.as_date("MMM dd, yyyy", Some("en_US_POSIX"), None)
		}
	}

	fn get_chapter_list(&self, id: String) -> Result<Vec<Chapter>> {
		let request = Request::get(format!(
			"{}/api/manga/{id}/chapters?source=detail",
			self.base_url()
		))
		.header("Referer", self.base_url());

		let document = request.html()?;
		let chapter_elems = document.select("#chapter-list > li");
		Ok(chapter_elems
			.array()
			.filter_map(|node| {
				if let Ok(node) = node.as_node() {
					let url = node.select("a").attr("abs:href").read();
					let id = url.trim_start_matches(self.base_url()).to_string();

					let title = node.select(".chapter-title").text().read();
					let date_updated =
						self.parse_chapter_date(node.select(".chapter-update").text());

					let numbers = extract_f32_from_string(title.clone());
					let (volume, chapter) = if title.to_lowercase().starts_with("vol") {
						(numbers[0], numbers[1])
					} else if !numbers.is_empty() {
						(-1.0, numbers[0])
					} else {
						(-1.0, -1.0)
					};

					Some(Chapter {
						id,
						volume,
						chapter,
						date_updated,
						title,
						url,
						..Default::default()
					})
				} else {
					None
				}
			})
			.collect::<Vec<_>>())
	}

	fn get_page_list(&self, _: String, id: String) -> Result<Vec<Page>> {
		let url = if id.starts_with("http") {
			// External chapter, so the id is the same as the URL
			id
		} else {
			format!("{}{id}", self.base_url())
		};
		let request = Request::get(url).header("Referer", self.base_url());
		let html = request.string()?;

		let page_list = html
			.substring_after("var chapImages = '")
			.expect("chapImages should exist if mainServer exist")
			.substring_before("'")
			.expect("chapImages should have closing quote")
			.split(',')
			.enumerate()
			.map(|(idx, v)| Page {
				index: idx as i32 + 1,
				url: v.to_string(),
				..Default::default()
			})
			.collect::<Vec<_>>();
		Ok(page_list)
	}

	fn modify_image_request(&self, request: Request) {
		request.header("Referer", self.base_url());
	}

	fn handle_url(&self, url: String) -> Result<DeepLink> {
		let segments = url.split('/').collect::<Vec<_>>();
		let last_index = segments.len() - 1;

		if segments[last_index].contains("chapter") {
			let manga_id = segments[last_index - 1].to_string();
			let id = segments[last_index].to_string();
			Ok(DeepLink {
				manga: self.get_manga_details(manga_id).ok(),
				chapter: Some(Chapter {
					id,
					..Default::default()
				}),
			})
		} else if segments.len() == 4 {
			let id = segments[last_index].to_string();
			Ok(DeepLink {
				manga: self.get_manga_details(id).ok(),
				chapter: None,
			})
		} else {
			Err(AidokuError {
				reason: AidokuErrorKind::Unimplemented,
			})
		}
	}

	fn handle_notification(&self, _: String) {
		todo!()
	}
}
