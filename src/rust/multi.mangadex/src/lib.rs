#![no_std]
#![feature(let_chains)]
extern crate alloc;
mod helper;
mod parser;
use aidoku::{
	error::*,
	prelude::*,
	std::{
		defaults::{defaults_get, defaults_set},
		net::{HttpMethod, Request},
		ArrayRef, ObjectRef, String, StringRef, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, Page,
};
use alloc::borrow::ToOwned;
use helper::*;

#[link(wasm_import_module = "net")]
extern "C" {
	fn set_rate_limit(rate_limit: i32);
	fn set_rate_limit_period(period: i32);
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn initialize() {
	set_rate_limit(3);
	set_rate_limit_period(1);

	for key in ["blockedGroups", "blockedUploaders"] {
		let arrkey = key.to_owned() + "Array";
		if let Ok(arr_val) = defaults_get(&arrkey) {
			if arr_val.as_array().is_err() {
				handle_notification(String::from(key));
			}
		}
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let offset = (page - 1) * 20;
	let mut url = String::from(
		"https://api.mangadex.org/manga/?includes[]=cover_art\
		&limit=20\
		&offset=",
	) + itoa::Buffer::new().format(offset);

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(value) = filter.value.as_string() {
					url.push_str("&title=");
					url.push_str(&urlencode(value.read()));
				}
			}
			FilterType::Author => {
				if let Ok(value) = filter.value.as_string() {
					url.push_str("&author=");
					url.push_str(&urlencode(value.read()));
				}
			}
			FilterType::Check => {
				let value = filter.value.as_int().unwrap_or(-1);
				if value < 0 {
					continue;
				}
				if let Ok(id) = filter.object.get("id").as_string() {
					let mut id = id.read();
					if value == 0 {
						id = id.replace("&originalLanguage", "&excludedOriginalLanguage");
					}
					url.push_str(&id);
				} else {
					match filter.name.as_str() {
						"Has available chapters" => {
							if value == 1 {
								url.push_str("&hasAvailableChapters=true");
								if let Ok(languages_value) = defaults_get("languages") {
									if let Ok(languages) = languages_value.as_array() {
										languages.for_each(|lang| {
											if let Ok(lang) = lang.as_string() {
												url.push_str("&availableTranslatedLanguage[]=");
												url.push_str(&lang.read());
											}
										})
									}
								}
							}
						}
						_ => continue,
					}
				}
			}
			FilterType::Genre => {
				if let Ok(id) = filter.object.get("id").as_string() {
					// Run `python scripts/update_tags.py` to fetch tags from https://api.mangadex.org/manga/tag
					match filter.value.as_int().unwrap_or(-1) {
						0 => url.push_str("&excludedTags[]="),
						1 => url.push_str("&includedTags[]="),
						_ => continue,
					}
					url.push_str(&id.read());
				}
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int().unwrap_or(0);
				let ascending = value.get("ascending").as_bool().unwrap_or(false);
				url.push_str("&order[");
				url.push_str(match index {
					0 => "latestUploadedChapter",
					1 => "relevance",
					2 => "followedCount",
					3 => "createdAt",
					4 => "updatedAt",
					5 => "title",
					_ => {
						// Un-push the last "&order[" pushed
						url.replace_range(url.len() - 7..url.len() - 1, "");
						continue;
					}
				});
				url.push_str("]=");
				url.push_str(if ascending { "asc" } else { "desc" });
			}
			FilterType::Select => match filter.name.as_str() {
				"Included tags mode" => {
					url.push_str("&includedTagsMode=");
					match filter.value.as_int().unwrap_or(-1) {
						0 => url.push_str("AND"),
						1 => url.push_str("OR"),
						_ => url.push_str("AND"),
					}
				}
				"Excluded tags mode" => {
					url.push_str("&excludedTagsMode=");
					match filter.value.as_int().unwrap_or(-1) {
						0 => url.push_str("AND"),
						1 => url.push_str("OR"),
						_ => url.push_str("OR"),
					}
				}
				_ => continue,
			},
			_ => continue,
		}
	}

	let json = Request::new(&url, HttpMethod::Get).json_rl().as_object()?;

	let data = json.get("data").as_array()?;

	let manga = data
		.filter_map(|manga| match manga.as_object() {
			Ok(obj) => parser::parse_basic_manga(obj).ok(),
			Err(_) => None,
		})
		.collect::<Vec<_>>();

	let total = json.get("total").as_int().unwrap_or(0) as i32;

	Ok(MangaPageResult {
		manga,
		has_more: offset + 20 < total,
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let mut filters: Vec<Filter> = Vec::with_capacity(1);
	let mut selection = ObjectRef::new();

	if listing.name == "Popular" {
		selection.set("index", 2.into());
		selection.set("ascending", false.into());
		filters.push(Filter {
			kind: FilterType::Sort,
			name: String::from("Sort"),
			value: selection.0.clone(),
			object: selection,
		});
	} else if listing.name == "Latest" {
		// get recently published chapters
		let offset = (page - 1) * 40;
		let mut url = String::from(
			"https://api.mangadex.org/chapter\
			?includes[]=manga\
			&order[publishAt]=desc\
			&includeFutureUpdates=0\
			&limit=40\
			&offset=",
		) + itoa::Buffer::new().format(offset);
		if let Ok(languages_value) = defaults_get("languages") {
			if let Ok(languages) = languages_value.as_array() {
				languages.for_each(|lang| {
					if let Ok(lang) = lang.as_string() {
						url.push_str("&translatedLanguage[]=");
						url.push_str(&lang.read());
					}
				})
			}
		}
		if let Ok(groups_value) = defaults_get("blockedGroupsArray") {
			if let Ok(groups) = groups_value.as_array() {
				groups.for_each(|group| {
					if let Ok(group) = group.as_string() {
						url.push_str("&excludedGroups[]=");
						url.push_str(&group.read());
					}
				});
			}
		}
		if let Ok(groups_value) = defaults_get("blockedUploadersArray") {
			if let Ok(groups) = groups_value.as_array() {
				groups.for_each(|group| {
					if let Ok(group) = group.as_string() {
						url.push_str("&excludedUploaders[]=");
						url.push_str(&group.read());
					}
				});
			}
		}

		let mut json = Request::new(&url, HttpMethod::Get).json_rl().as_object()?;

		let total = json.get("total").as_int().unwrap_or(0) as i32;
		let mut data = json.get("data").as_array()?;

		let manga_ids = data
			.filter_map(|chapter| match chapter.as_object() {
				Ok(obj) => {
					if let Ok(relationships) = obj.get("relationships").as_array() {
						for relationship in relationships {
							if let Ok(relationship) = relationship.as_object()
								   && let Ok(relation_type) = relationship.get("type").as_string()
								   && relation_type.read() == "manga"
								   && let Ok(id) = relationship.get("id").as_string() {
									let mut ret = String::from("&ids[]=");
									ret.push_str(&id.read());
									return Some(ret);
								}
						}
						None
					} else {
						None
					}
				}
				Err(_) => None,
			})
			.collect::<String>();

		url = String::from(
			"https://api.mangadex.org/manga\
			?includes[]=cover_art\
			&order[updatedAt]=desc\
			&contentRating[]=erotica\
			&contentRating[]=suggestive\
			&contentRating[]=safe",
		) + &manga_ids;
		json = Request::new(&url, HttpMethod::Get).json_rl().as_object()?;
		data = json.get("data").as_array()?;
		let manga = data
			.filter_map(|manga| match manga.as_object() {
				Ok(obj) => parser::parse_basic_manga(obj).ok(),
				Err(_) => None,
			})
			.collect::<Vec<_>>();

		return Ok(MangaPageResult {
			manga,
			has_more: offset + 20 < total,
		});
	}

	get_manga_list(filters, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = String::from("https://api.mangadex.org/manga/")
		+ &id + "?includes[]=cover_art\
		&includes[]=author\
		&includes[]=artist";
	let json = Request::new(&url, HttpMethod::Get).json_rl().as_object()?;

	let data = json.get("data").as_object()?;

	parser::parse_full_manga(data)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let mut url = String::from("https://api.mangadex.org/manga/")
		+ &id + "/feed\
		?order[volume]=desc\
		&order[chapter]=desc\
		&limit=500\
		&contentRating[]=pornographic\
		&contentRating[]=erotica\
		&contentRating[]=suggestive\
		&contentRating[]=safe\
		&includes[]=user\
		&includes[]=scanlation_group";

	if let Ok(languages_value) = defaults_get("languages") {
		if let Ok(languages) = languages_value.as_array() {
			languages.for_each(|lang| {
				if let Ok(lang) = lang.as_string() {
					url.push_str("&translatedLanguage[]=");
					url.push_str(&lang.read());
				}
			})
		}
	}
	if let Ok(groups_value) = defaults_get("blockedGroupsArray") {
		if let Ok(groups) = groups_value.as_array() {
			groups.for_each(|group| {
				if let Ok(group) = group.as_string() {
					url.push_str("&excludedGroups[]=");
					url.push_str(&group.read());
				}
			});
		}
	}
	if let Ok(groups_value) = defaults_get("blockedUploadersArray") {
		if let Ok(groups) = groups_value.as_array() {
			groups.for_each(|group| {
				if let Ok(group) = group.as_string() {
					url.push_str("&excludedUploaders[]=");
					url.push_str(&group.read());
				}
			});
		}
	}
	let json = Request::new(&url, HttpMethod::Get).json_rl().as_object()?;
	let total = json.get("total").as_int().unwrap_or(0);
	let data = json.get("data").as_array()?;
	let mut chapters: Vec<Chapter> = Vec::with_capacity(total.try_into().unwrap_or(0));
	chapters.append(
		&mut data
			.filter_map(|chapter| match chapter.as_object() {
				Ok(obj) => parser::parse_chapter(obj).ok(),
				Err(_) => None,
			})
			.collect::<Vec<_>>(),
	);

	let mut offset = 500;
	while offset < total {
		let json = Request::new(
			&(url.clone() + "&offset=" + itoa::Buffer::new().format(offset)),
			HttpMethod::Get,
		)
		.json_rl();

		if let Ok(json) = json.as_object() {
			let data = json.get("data").as_array()?;
			chapters.append(
				&mut data
					.filter_map(|chapter| match chapter.as_object() {
						Ok(obj) => parser::parse_chapter(obj).ok(),
						Err(_) => None,
					})
					.collect::<Vec<_>>(),
			);
		}
		offset += 500;
	}
	Ok(chapters)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut url = String::from("https://api.mangadex.org/at-home/server/") + &chapter_id;
	if let Ok(port_value) = defaults_get("standardHttpsPort") {
		if port_value.as_bool().unwrap_or(false) {
			url.push_str("?forcePort443=true");
		}
	}
	let json = Request::new(&url, HttpMethod::Get).json_rl().as_object()?;

	let chapter = json.get("chapter").as_object()?;
	let data_saver = match defaults_get("dataSaver") {
		Ok(data_saver) => data_saver.as_bool().unwrap_or(false),
		Err(_) => false,
	};
	let data = chapter
		.get(if data_saver { "dataSaver" } else { "data" })
		.as_array()?;

	let base_url = json.get("baseUrl").as_string()?.read();
	let hash = chapter.get("hash").as_string()?.read();
	let path = if data_saver {
		String::from("/data-saver/")
	} else {
		String::from("/data/")
	};

	Ok(data
		.enumerate()
		.filter_map(|(i, page)| match page.as_string() {
			Ok(data) => {
				let data = data.read();
				let mut url = String::with_capacity(
					base_url.len() + hash.len() + data.len() + path.len() + 1,
				);
				url.push_str(&base_url);
				url.push_str(&path);
				url.push_str(&hash);
				url.push('/');
				url.push_str(&data);

				Some(Page {
					index: i as i32,
					url,
					base64: String::new(),
					text: String::new(),
				})
			}
			Err(_) => None,
		})
		.collect::<Vec<_>>())
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let url = &url[21..]; // remove "https://mangadex.org/"

	if url.starts_with("title") {
		// ex: https://mangadex.org/title/a96676e5-8ae2-425e-b549-7f15dd34a6d8/komi-san-wa-komyushou-desu
		let id = &url[6..]; // remove "title/"
		let end = id.find('/').unwrap_or(id.len());
		let manga_id = &id[..end];

		return Ok(DeepLink {
			manga: get_manga_details(String::from(manga_id)).ok(),
			chapter: None,
		});
	} else if url.starts_with("chapter") {
		// ex: https://mangadex.org/chapter/56eecc6f-1a4e-464c-b6a4-a1cbdfdfd726/1
		let id = &url[8..]; // remove "chapter/"
		let end = id.find('/').unwrap_or(id.len());
		let chapter_id = &id[..end];

		let url = String::from("https://api.mangadex.org/chapter/") + chapter_id;

		let json = Request::new(&url, HttpMethod::Get).json_rl().as_object()?;

		let chapter_obj = json.get("data").as_object()?;
		let relationships = chapter_obj.get("relationships").as_array()?;
		for relationship in relationships {
			if let Ok(obj) = relationship.as_object()
				&& let Ok(relation_type) = obj.get("type").as_string()
				&& relation_type.read() == "manga"
				&& let Ok(manga_id) = obj.get("id").as_string()
			{
				return Ok(DeepLink {
					manga: get_manga_details(manga_id.read()).ok(),
					chapter: parser::parse_chapter(chapter_obj).ok(),
				})
			}
		}
	}

	Err(aidoku::error::AidokuError {
		reason: aidoku::error::AidokuErrorKind::Unimplemented,
	})
}

#[handle_notification]
fn handle_notification(notification: String) {
	match notification.as_str() {
		"blockedGroups" | "blockedUploaders" => {
			if let Ok(groups) = defaults_get(&notification) {
				if let Ok(groups_string) = groups.as_string() {
					let mut arr = ArrayRef::new();
					groups_string.read().split(',').for_each(|group| {
						let trimmed = group.trim();
						if !trimmed.is_empty() {
							arr.insert(StringRef::from(trimmed).0);
						}
					});
					defaults_set((notification + "Array").as_str(), arr.0);
				}
			}
		}
		_ => {}
	}
}
