#![no_std]
use aidoku::{
	prelude::*, error::Result, std::String, std::Vec, std::ArrayRef, std::net::Request, std::net::HttpMethod,
	Filter, FilterType, Listing, Manga, MangaPageResult, Page, MangaStatus, MangaContentRating, MangaViewer, Chapter, DeepLink,
	std::defaults::defaults_get,
};

mod helper;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();
	let mut sort = String::new();
	let mut included_tags: Vec<String> = Vec::new();
	let mut excluded_tags: Vec<String> = Vec::new();
	let mut types: Vec<String> = Vec::new();
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = helper::urlencode(filter.value.as_string()?.read());
			},
			FilterType::Genre => {
				if let Ok(tag_id) = filter.object.get("id").as_string() {
					match filter.value.as_int().unwrap_or(-1) {
						0 => excluded_tags.push(tag_id.read()),
						1 => included_tags.push(tag_id.read()),
						_ => continue,
					}
				}
			},
			FilterType::Check => {
				if filter.value.as_int().unwrap_or(-1) <= 0 {
					continue;
				}
				types.push(filter.name);
			},
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int().unwrap_or(0);
				let option = match index {
					0 => "",
					1 => "name",
					2 => "created_at",
					_ => "",
				};
				sort = String::from(option)
			},
			_ => continue,
		}
	}

	let mut manga_arr: Vec<Manga> = Vec::new();
	let mut total: i32 = 1;

	let mut url = format!("https://dynasty-scans.com/search?q={}&sort={}&page={}", query, sort, helper::i32_to_string(page));
	if !included_tags.is_empty() {
		for tag in included_tags {
			url.push_str("&with%5B%5D=");
			url.push_str(&tag);
		}
	}
	if !excluded_tags.is_empty() {
		for tag in excluded_tags {
			url.push_str("&without%5B%5D=");
			url.push_str(&tag);
		}
	}
	if !types.is_empty() {
		for type_name in types {
			url.push_str("&classes%5B%5D=");
			url.push_str(&type_name);
		}
	} else {
		url.push_str("&classes%5B%5D=Series");
	}

	let skip_images = defaults_get("skipImages").as_bool().unwrap_or(false);

	let html = Request::new(url.as_str(), HttpMethod::Get).html();
	for result in html.select(".chapter-list a.name").array() {
		let result_node = result.as_node();
		let manga_url = result_node.attr("href").read();
		if manga_url.is_empty() {
			continue;
		}
		if skip_images {
			let title = result_node.text().read();
			manga_arr.push(Manga {
				id: String::from(&manga_url[1..]),
				cover: String::new(),
				title,
				author: String::new(),
				artist: String::new(),
				description: String::new(),
				url: String::new(),
				categories: Vec::new(),
				status: MangaStatus::Completed,
				nsfw: MangaContentRating::Nsfw,
				viewer: MangaViewer::Rtl
			});
		} else {
			match helper::get_manga_details(String::from(&manga_url[1..])) {
				Ok(manga) => {
					manga_arr.push(manga);
				},
				Err(_) => continue,
			}
		}
	}

	for page in html.select("div.pagination a").array() {
		let text = page.as_node().text().read();
		if let Ok(num) = text.parse::<i32>() {
			if num > total {
				total = num;
			}
		}
	}

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: page < total,
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	match listing.name.as_str() {
		"Recently Added" => {},
		_ => return get_manga_list(Vec::new(), page)
	}

	let mut added_ids: Vec<String> = Vec::new();
	let mut manga_arr: Vec<Manga> = Vec::new();
	
	let skip_images = defaults_get("skipImages").as_bool().unwrap_or(false);

	let json = Request::new(format!("https://dynasty-scans.com/chapters/added.json?page={}", &helper::i32_to_string(page)).as_str(), HttpMethod::Get).json().as_object()?;
	for chapter in json.get("chapters").as_array()? {
		let chapter_object = chapter.as_object()?;
		let result_object;
		let tags = chapter_object.get("tags").as_array()?;
		let series = helper::find_in_array(tags.clone(), String::from("Series"))?;
		if series.len() > 0 {
			result_object = series[0].clone();
		} else { // anthology or doujin
			let anthologies = helper::find_in_array(tags.clone(), String::from("Anthology"))?;
			if anthologies.len() > 0 {
				result_object = anthologies[0].clone();
			} else { // has to be a doujin
				let doujins = helper::find_in_array(tags.clone(), String::from("Doujin"))?;
				if doujins.len() > 0 {
					result_object = doujins[0].clone();
				} else { // idek
					continue;
				}
			}
		}
		let mut id = String::from(match result_object.get("type").as_string()?.read().as_str() {
			"Series" => "series",
			"Anthology" => "anthologies",
			"Doujin" => "doujins",
			_ => continue
		});
		id.push_str("/");
		id.push_str(result_object.get("permalink").as_string()?.read().as_str());
		if added_ids.contains(&id.clone()) {
			continue;
		}
		added_ids.push(id.clone());
		if skip_images {
			let title = result_object.get("name").as_string()?.read();
			manga_arr.push(Manga {
				id: id.clone(),
				cover: String::new(),
				title,
				author: String::new(),
				artist: String::new(),
				description: String::new(),
				url: String::new(),
				categories: Vec::new(),
				status: MangaStatus::Completed,
				nsfw: MangaContentRating::Nsfw,
				viewer: MangaViewer::Rtl
			});
		} else {
			match helper::get_manga_details(id.clone()) {
				Ok(manga) => {
					manga_arr.push(manga);
				},
				Err(_) => continue,
			}
		}
	}
	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: page < json.get("total_pages").as_int().unwrap_or(0) as i32,
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	helper::get_manga_details(id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("https://dynasty-scans.com/{}.json", &id);
	let json = Request::new(url.as_str(), HttpMethod::Get).json().as_object()?;
	
	let json_chapters = json.get("taggings").as_array()?;
	let mut chapters = Vec::new();

	let mut volume_on: f32 = -1.0;
	for chapter in json_chapters {
		let chapter_obj = chapter.as_object()?;

		if let Ok(header) = chapter_obj.get("header").as_string() {
			volume_on = helper::string_after(header.read(), ' ').parse::<f32>().unwrap_or(-1.0);
			continue;
		}

		let chapter_id = chapter_obj.get("permalink").as_string()?.read();
		
		let title = match chapter_obj.get("title").as_string() {
			Ok(title) => title.read(),
			Err(_) => String::new()
		};
		
		let chapter_url = format!("https://dynasty-scans.com/chapters/{}", chapter_id.clone());
		let date_updated = match chapter_obj.get("released_on").as_date("YYYY-MM-dd", None, None) {
			Ok(date_updated) => date_updated,
			Err(_) => 0.0
		};
		let chapter_num_pos = id.split("/").last().unwrap().len()+3;
		let chapter_num = if chapter_num_pos >= chapter_id.len() { -1.0 } else { helper::string_replace(String::from(&chapter_id[chapter_num_pos..]), '_', '.').parse::<f32>().unwrap_or(-1.0) };
		
		let tags = match chapter_obj.get("tags").as_array() {
			Ok(tags) => tags,
			Err(_) => ArrayRef::new()
		};
		let scanlator = match helper::find_in_array(tags, String::from("Scanlator")) {
			Ok(scanlator_arr) => if scanlator_arr.len() > 0 { scanlator_arr[0].get("name").as_string()?.read() } else { String::new() },
			Err(_) => String::new()
		};

		chapters.push(Chapter {
			id: chapter_id,
			title,
			volume: volume_on,
			chapter: chapter_num,
			date_updated,
			scanlator,
			url: chapter_url,
			lang: String::from("en"),
		});
	}

	Ok(chapters.into_iter().rev().collect())
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	let url = format!("https://dynasty-scans.com/chapters/{}.json", &id);
	let json = Request::new(url.as_str(), HttpMethod::Get).json().as_object()?;

	let pages_arr = json.get("pages").as_array()?;

	let mut pages = Vec::new();
	let mut index = 0;

	for page in pages_arr {
		let page_obj = page.as_object()?;
		let url = format!("https://dynasty-scans.com{}", page_obj.get("url").as_string()?.read());

		pages.push(Page {
			index,
			url,
			base64: String::new(),
			text: String::new()
		});

		index += 1;
	}

	Ok(pages)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let mut count = 0;
	let mut index = 0;
	for (i, c) in url.chars().enumerate() {
		if count == 3 {
			index = i;
			break;
		}
		if c == '/' {
			count += 1;
		}
	}
	let manga_id = &url[index..];
	let manga = get_manga_details(String::from(manga_id))?;
	return Ok(DeepLink {
		manga: Some(manga),
		chapter: None,
	});
}
