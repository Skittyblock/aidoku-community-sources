#![no_std]

pub use aidoku::{
	error::Result,
	prelude::*,
	std::{String, Vec},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, Page,
};

use aidoku::{
	error::{AidokuError, AidokuErrorKind},
	helpers::uri::{encode_uri_component, QueryParameters},
	std::net::{HttpMethod, Request},
	std::{ArrayRef, ObjectRef},
	MangaContentRating, MangaStatus, MangaViewer,
};

const USER_AGENT: &str = "Mozilla/5.0 (iPhone; like Mac OS X) Aidoku/0.1";

static NSFW_CATEGORIES: [&str; 2] = ["Hentai", "Smut"];

static SUGGESTIVE_CATEGORIES: [&str; 1] = ["Ecchi"];

macro_rules! get_value {
	($obj:ident, $key:tt, $as:tt) => {
		$obj.get(stringify!($key)).$as().unwrap()
	};

	($obj:ident, $key:tt, $as:tt, $or:expr) => {
		$obj.get(stringify!($key)).$as().unwrap_or($or)
	};
}

macro_rules! get_array_as_vec {
	($obj:ident, $key:tt) => {
		get_value!($obj, $key, as_array)
			.map(|it| it.as_string().unwrap().read())
			.collect::<Vec<String>>()
	};
}

macro_rules! push_sort_param {
	($params:ident, $asc:expr, $name:literal) => {
		$params.push_encoded("sort", Some(if $asc { $name } else { concat!('-', $name) }))
	};
}

#[inline]
fn vec_from_array<T>(array: &ArrayRef) -> Vec<T> {
	Vec::<T>::with_capacity(array.len())
}

#[inline]
fn vec_intersects(a: &[&str], b: &[String]) -> bool {
	b.iter().any(|it| a.contains(&it.as_str()))
}

#[inline]
fn json_request(url: String) -> ObjectRef {
	// TODO: handle possible errors
	Request::new(url, HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.json()
		.unwrap()
		.as_object()
		.unwrap()
}

pub struct MangAdventure {
	pub base_url: &'static str,
	pub language: &'static str,
}

impl MangAdventure {
	fn get_manga_page_result(&self, url: String) -> Result<MangaPageResult> {
		let json = json_request(url);
		let last = get_value!(json, last, as_bool);
		let results = get_value!(json, results, as_array);
		let mut manga = vec_from_array::<Manga>(&results);

		for result in results {
			let obj = result.as_object().unwrap();
			// exclude licensed series ("chapters": null)
			if obj.get("chapters").is_none() { continue; }
			let mut url = get_value!(obj, url, as_string).read();
			url.insert_str(0, self.base_url);
			let id = get_value!(obj, slug, as_string).read();
			let title = get_value!(obj, title, as_string).read();
			let cover = get_value!(obj, cover, as_string).read();
			manga.push(Manga {
				id,
				url,
				title,
				cover,
				author: String::new(),
				artist: String::new(),
				description: String::new(),
				categories: Vec::new(),
				status: MangaStatus::Unknown,
				nsfw: MangaContentRating::Safe,
				viewer: MangaViewer::Rtl,
			});
		}

		Ok(MangaPageResult {
			manga,
			has_more: !last,
		})
	}

	pub fn get_manga_list(&self, filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
		let mut categories = Vec::<String>::new();
		let mut params = QueryParameters::new();

		for filter in filters {
			match filter.kind {
				FilterType::Title => {
					if let Ok(value) = filter.value.as_string() {
						params.push(String::from("title"), Some(value.read()));
					}
				}
				FilterType::Author => {
					// TODO: add artist search when supported
					if let Ok(value) = filter.value.as_string() {
						params.push(String::from("author"), Some(value.read()));
					}
				}
				FilterType::Select => match filter.value.as_int().unwrap_or(0) {
					0 => params.push_encoded("status", Some("any")),
					1 => params.push_encoded("status", Some("completed")),
					2 => params.push_encoded("status", Some("ongoing")),
					3 => params.push_encoded("status", Some("hiatus")),
					4 => params.push_encoded("status", Some("canceled")),
					_ => continue,
				},
				FilterType::Sort => {
					if let Ok(value) = filter.value.as_object() {
						let asc = get_value!(value, ascending, as_bool, false);
						match get_value!(value, index, as_int, 0) {
							0 => push_sort_param!(params, asc, "title"),
							1 => push_sort_param!(params, asc, "views"),
							2 => push_sort_param!(params, asc, "latest_upload"),
							3 => push_sort_param!(params, asc, "chapter_count"),
							_ => continue,
						}
					}
				}
				FilterType::Genre => {
					let mut id = encode_uri_component(filter.name.to_lowercase());
					match filter.value.as_int().unwrap_or(-1) {
						0 => {
							id.insert(0, '-');
							categories.push(id);
						}
						1 => categories.push(id),
						_ => continue,
					}
				}
				_ => continue,
			}
		}

		if !categories.is_empty() {
			params.push_encoded(String::from("categories"), Some(categories.join(",")));
		}

		self.get_manga_page_result(format!(
			"{}/api/v2/series?page={}&{}",
			self.base_url, page, params
		))
	}

	pub fn get_manga_listing(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		let mut url = format!("{}/api/v2/series?page={}", self.base_url, page);

		match listing.name.as_str() {
			"Most Viewed" => url.push_str("&sort=-views"),
			"Latest Updates" => url.push_str("&sort=-latest_upload"),
			_ => url.push_str("&sort=title"),
		}

		self.get_manga_page_result(url)
	}

	pub fn get_manga_details(&self, id: String) -> Result<Manga> {
		let json = json_request(format!("{}/api/v2/series/{}", self.base_url, id));
		let mut url = get_value!(json, url, as_string).read();
		url.insert_str(0, self.base_url);
		let id = get_value!(json, slug, as_string).read();
		let title = get_value!(json, title, as_string).read();
		let cover = get_value!(json, cover, as_string).read();
		let description = get_value!(json, description, as_string).read();
		let author = get_array_as_vec!(json, authors).join(", ");
		let artist = get_array_as_vec!(json, artists).join(", ");
		let categories = get_array_as_vec!(json, categories);
		let status = match get_value!(json, status, as_string).read().as_str() {
			"completed" => MangaStatus::Completed,
			"ongoing" => MangaStatus::Ongoing,
			"hiatus" => MangaStatus::Hiatus,
			"canceled" => MangaStatus::Cancelled,
			_ => MangaStatus::Unknown,
		};
		let nsfw = if vec_intersects(&NSFW_CATEGORIES, &categories) {
			MangaContentRating::Nsfw
		} else if vec_intersects(&SUGGESTIVE_CATEGORIES, &categories) {
			MangaContentRating::Suggestive
		} else {
			MangaContentRating::Safe
		};

		Ok(Manga {
			id,
			url,
			title,
			cover,
			author,
			artist,
			description,
			categories,
			status,
			nsfw,
			viewer: MangaViewer::Rtl,
		})
	}

	pub fn get_chapter_list(&self, id: String) -> Result<Vec<Chapter>> {
		let json = json_request(format!(
			"{}/api/v2/series/{}/chapters?date_format=timestamp",
			self.base_url, id
		));
		let results = get_value!(json, results, as_array);
		let mut chapters = vec_from_array::<Chapter>(&results);

		for result in results {
			let obj = result.as_object().unwrap();
			let mut url = get_value!(obj, url, as_string).read();
			url.insert_str(0, self.base_url);
			let title = get_value!(obj, title, as_string).read();
			let chapter = get_value!(obj, number, as_float) as f32;
			let volume = get_value!(obj, volume, as_int, -1) as f32;
			let scanlator = get_array_as_vec!(obj, groups).join(", ");
			let date_updated = get_value!(obj, published, as_string)
				.read()
				.parse::<f64>()
				.unwrap() / 1e3;

			chapters.push(Chapter {
				url,
				title,
				volume,
				chapter,
				scanlator,
				date_updated,
				lang: String::from(self.language),
				id: format!("{}", get_value!(obj, id, as_int)),
			});
		}

		Ok(chapters)
	}

	pub fn get_page_list(&self, id: String) -> Result<Vec<Page>> {
		let json = json_request(format!(
			"{}/api/v2/chapters/{}/pages?track=true",
			self.base_url, id
		));
		let results = get_value!(json, results, as_array);
		let mut pages = vec_from_array::<Page>(&results);

		for result in results {
			let obj = result.as_object().unwrap();
			let url = get_value!(obj, image, as_string).read();
			let index = get_value!(obj, number, as_int) as i32 - 1;

			pages.push(Page {
				url,
				index,
				text: String::new(),
				base64: String::new(),
			});
		}

		Ok(pages)
	}

	pub fn handle_url(&self, url: String) -> Result<DeepLink> {
		let parts = url.split('/').collect::<Vec<&str>>();
		if let Some(top) = parts.get(3) {
			if top == &"reader" {
				if let Some(slug) = parts.get(4) {
					let manga = self.get_manga_details(String::from(*slug)).ok();
					return Ok(DeepLink {
						manga,
						chapter: None,
					});
					// TODO: implement chapter deeplink
				}
			}
		}

		Err(AidokuError {
			reason: AidokuErrorKind::Unimplemented,
		})
	}
}
