#![no_std]
#![feature(let_chains)]
extern crate alloc;
mod helper;
mod parser;
use aidoku::{
	error::*,
	prelude::*,
	std::{
		defaults::defaults_get,
		net::{HttpMethod, Request},
		ObjectRef, String, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, Page,
};
use alloc::string::ToString;
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
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let offset = (page - 1) * 20;
	let mut url = String::from(
		"https://api.mangadex.org/manga/?includes[]=cover_art\
		&limit=20\
		&offset=",
	);
	url.push_str(&offset.to_string());

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
								if let Ok(languages) = defaults_get("languages").as_array() {
									for lang in languages {
										if let Ok(lang) = lang.as_string() {
											url.push_str("&availableTranslatedLanguage[]=");
											url.push_str(&lang.read());
										}
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
					_ => continue,
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
		.map(|manga| {
			let obj = manga.as_object()?;
			parser::parse_basic_manga(obj)
		})
		.filter_map(|manga| manga.ok())
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
		);
		url.push_str(&offset.to_string());
		if let Ok(languages) = defaults_get("languages").as_array() {
			for lang in languages {
				if let Ok(lang) = lang.as_string() {
					url.push_str("&translatedLanguage[]=");
					url.push_str(&lang.read());
				}
			}
		}
		if let Ok(groups_string) = defaults_get("blockedGroups").as_string() {
			groups_string.read().split(',').for_each(|group| {
				let trimmed = group.trim();
				if !trimmed.is_empty() {
					url.push_str("&excludedGroups[]=");
					url.push_str(trimmed);
				}
			});
		}
		if let Ok(groups_string) = defaults_get("blockedUploaders").as_string() {
			groups_string.read().split(',').for_each(|group| {
				let trimmed = group.trim();
				if !trimmed.is_empty() {
					url.push_str("&excludedUploaders[]=");
					url.push_str(trimmed);
				}
			});
		}

		let mut json = Request::new(&url, HttpMethod::Get).json_rl().as_object()?;

		let total = json.get("total").as_int().unwrap_or(0) as i32;
		let mut data = json.get("data").as_array()?;

		let manga_ids = data
			.map(|chapter| {
				let obj = chapter.as_object()?;
				let relationships = obj.get("relationships").as_array()?;
				for relationship in relationships {
					let relationship = relationship.as_object()?;
					let relation_type = relationship.get("type").as_string()?.read();
					if relation_type == "manga" {
						let mut ret = String::from("&ids[]=");
						ret.push_str(&relationship.get("id").as_string()?.read());
						return Ok(ret);
					}
				}
				Err(AidokuError {
					reason: AidokuErrorKind::Unimplemented,
				})
			})
			.filter_map(|id| id.ok())
			.collect::<String>();

		url = String::from(
			"https://api.mangadex.org/manga\
			?includes[]=cover_art\
			&order[updatedAt]=desc\
			&contentRating[]=erotica\
			&contentRating[]=suggestive\
			&contentRating[]=safe",
		);
		url.push_str(&manga_ids);
		json = Request::new(&url, HttpMethod::Get).json_rl().as_object()?;
		data = json.get("data").as_array()?;
		let manga = data
			.map(|manga| {
				let obj = manga.as_object()?;
				parser::parse_basic_manga(obj)
			})
			.filter_map(|manga| manga.ok())
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
	let mut url = String::from("https://api.mangadex.org/manga/");
	url.push_str(&id);
	url.push_str(
		"?includes[]=cover_art\
		&includes[]=author\
		&includes[]=artist",
	);
	let json = Request::new(&url, HttpMethod::Get).json_rl().as_object()?;

	let data = json.get("data").as_object()?;

	parser::parse_full_manga(data)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let mut url = String::from("https://api.mangadex.org/manga/");
	url.push_str(&id);
	url.push_str(
		"/feed\
		?order[volume]=desc\
		&order[chapter]=desc\
		&limit=500\
		&contentRating[]=pornographic\
		&contentRating[]=erotica\
		&contentRating[]=suggestive\
		&contentRating[]=safe\
		&includes[]=scanlation_group",
	);
	if let Ok(languages) = defaults_get("languages").as_array() {
		for lang in languages {
			if let Ok(lang) = lang.as_string() {
				url.push_str("&translatedLanguage[]=");
				url.push_str(&lang.read());
			}
		}
	}

	if let Ok(groups_string) = defaults_get("blockedGroups").as_string() {
		groups_string.read().split(',').for_each(|group| {
			let trimmed = group.trim();
			if !trimmed.is_empty() {
				url.push_str("&excludedGroups[]=");
				url.push_str(trimmed);
			}
		});
	}
	if let Ok(groups_string) = defaults_get("blockedUploaders").as_string() {
		groups_string.read().split(',').for_each(|group| {
			let trimmed = group.trim();
			if !trimmed.is_empty() {
				url.push_str("&excludedUploaders[]=");
				url.push_str(trimmed);
			}
		});
	}
	let json = Request::new(&url, HttpMethod::Get).json_rl().as_object()?;
	let total = json.get("total").as_int().unwrap_or(0);
	let data = json.get("data").as_array()?;
	let mut chapters: Vec<Chapter> = Vec::with_capacity(total.try_into().unwrap_or(0));
	chapters.append(
		&mut data
			.map(|chapter| {
				let chapter_obj = chapter.as_object()?;
				parser::parse_chapter(chapter_obj)
			})
			.filter_map(|chapter| chapter.ok())
			.collect::<Vec<_>>(),
	);

	let mut offset = 500;
	while offset < total {
		let json = Request::new(
			&{
				let mut url = url.clone();
				url.push_str("&offset=");
				url.push_str(&offset.to_string());
				url
			},
			HttpMethod::Get,
		)
		.json_rl();

		if let Ok(json) = json.as_object() {
			let data = json.get("data").as_array()?;
			chapters.append(
				&mut data
					.map(|chapter| {
						let chapter_obj = chapter.as_object()?;
						parser::parse_chapter(chapter_obj)
					})
					.filter_map(|chapter| chapter.ok())
					.collect::<Vec<_>>(),
			);
		}
		offset += 500;
	}
	Ok(chapters)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	let mut url = String::from("https://api.mangadex.org/at-home/server/");
	url.push_str(&id);
	if defaults_get("standardHttpsPort").as_bool().unwrap_or(false) {
		url.push_str("?forcePort443=true");
	}
	let json = Request::new(&url, HttpMethod::Get).json_rl().as_object()?;

	let chapter = json.get("chapter").as_object()?;
	let data = chapter
		.get(if defaults_get("dataSaver").as_bool().unwrap_or(false) {
			"dataSaver"
		} else {
			"data"
		})
		.as_array()?;

	let base_url = json.get("baseUrl").as_string()?.read();
	let hash = chapter.get("hash").as_string()?.read();
	let path = if defaults_get("dataSaver").as_bool().unwrap_or(false) {
		String::from("/data-saver/")
	} else {
		String::from("/data/")
	};

	Ok(data
		.enumerate()
		.map(|(i, page)| {
			let data = page.as_string()?.read();
			let mut url =
				String::with_capacity(base_url.len() + hash.len() + data.len() + path.len() + 1);
			url.push_str(&base_url);
			url.push_str(&path);
			url.push_str(&hash);
			url.push('/');
			url.push_str(&data);

			Ok(Page {
				index: i as i32,
				url,
				base64: String::new(),
				text: String::new(),
			})
		})
		.filter_map(|page: Result<Page>| page.ok())
		.collect::<Vec<_>>())
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let url = &url[21..]; // remove "https://mangadex.org/"

	if url.starts_with("title") {
		// ex: https://mangadex.org/title/a96676e5-8ae2-425e-b549-7f15dd34a6d8/komi-san-wa-komyushou-desu
		let id = &url[6..]; // remove "title/"
		let end = match id.find('/') {
			Some(end) => end,
			None => id.len(),
		};
		let manga_id = &id[..end];

		return Ok(DeepLink {
			manga: get_manga_details(String::from(manga_id)).ok(),
			chapter: None,
		});
	} else if url.starts_with("chapter") {
		// ex: https://mangadex.org/chapter/56eecc6f-1a4e-464c-b6a4-a1cbdfdfd726/1
		let id = &url[8..]; // remove "chapter/"
		let end = match id.find('/') {
			Some(end) => end,
			None => id.len(),
		};
		let chapter_id = &id[..end];

		let mut url = String::from("https://api.mangadex.org/chapter/");
		url.push_str(chapter_id);

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
