use aidoku::{
	error::Result,
	prelude::format,
	std::{html::unescape_html_entities, net::HttpMethod, net::Request, String, Vec},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

extern crate alloc;
use crate::helper::*;

pub fn parse_manga_list(
	api_url: String,
	filters: Vec<Filter>,
	page: i32,
) -> Result<MangaPageResult> {
	let mut included_tags: Vec<String> = Vec::new();
	let mut excluded_tags: Vec<String> = Vec::new();
	let mut demographic_tags: Vec<String> = Vec::new();
	let mut sort_by: String = String::new();
	let mut title: String = String::new();
	let mut manga_type: String = String::new();
	let mut completed: String = String::new();
	let sort_options = [
		"",
		"view",
		"uploaded",
		"rating",
		"follow",
		"user_follow_count",
	];
	let type_options = ["", "jp", "kr", "cn"];
	let completed_options = ["", "true", "false"];
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
				let index = filter.value.as_int().unwrap_or(-1) as usize;
				match filter.name.as_str() {
					"Sort" => sort_by = String::from(sort_options[index]),
					"Type" => manga_type = String::from(type_options[index]),
					"Completed" => completed = String::from(completed_options[index]),
					_ => continue,
				}
			}
			FilterType::Check => {
				if filter.value.as_int().unwrap_or(-1) <= 0 {
					continue;
				}
				match filter.name.as_str() {
					"Shounen" => demographic_tags.push(String::from("1")),
					"Shoujo" => demographic_tags.push(String::from("2")),
					"Seinen" => demographic_tags.push(String::from("3")),
					"Josei" => demographic_tags.push(String::from("4")),
					_ => continue,
				}
			}
			_ => continue,
		};
	}

	if title.is_empty()
		&& demographic_tags.is_empty()
		&& included_tags.is_empty()
		&& excluded_tags.is_empty()
		&& sort_by.is_empty()
		&& manga_type.is_empty()
		&& completed.is_empty()
	{
		parse_manga_listing(api_url, String::from("Hot"), page)
	} else {
		let url = get_search_url(
			api_url,
			title,
			included_tags,
			excluded_tags,
			demographic_tags,
			manga_type,
			sort_by,
			completed,
			page,
		);
		let mut mangas: Vec<Manga> = Vec::new();
		let json = Request::new(url, HttpMethod::Get).json()?.as_array()?;
		let has_more = !json.is_empty();
		for data in json {
			if let Ok(data_obj) = data.as_object() {
				let title = match data_obj.get("title").as_string() {
					Ok(node) => node.read(),
					Err(_) => continue,
				};
				let mut id = match data_obj.get("slug").as_string() {
					Ok(node) => node.read(),
					Err(_) => continue,
				};

				id += "|";
				id += &(match data_obj.get("hid").as_string() {
					Ok(node) => node.read(),
					Err(_) => continue,
				});
				let cover = match data_obj.get("cover_url").as_string() {
					Ok(node) => node.read(),
					Err(_) => continue,
				};
				mangas.push(Manga {
					id: id.clone(),
					cover,
					title,
					author: String::new(),
					artist: String::new(),
					description: String::new(),
					url: String::new(),
					categories: Vec::new(),
					status: MangaStatus::Unknown,
					nsfw: MangaContentRating::Safe,
					viewer: MangaViewer::Rtl,
				});
			}
		}
		Ok(MangaPageResult {
			manga: mangas,
			has_more,
		})
	}
}

pub fn parse_manga_listing(
	api_url: String,
	list_type: String,
	page: i32,
) -> Result<MangaPageResult> {
	let url = get_listing_url(api_url.clone(), list_type, page);
	let mut mangas: Vec<Manga> = Vec::new();
	let json = Request::new(url, HttpMethod::Get).json()?.as_array()?;

	for data in json {
		if let Ok(data_obj) = data.as_object() {
			if let Ok(manga_obj) = data_obj.get("md_comics").as_object() {
				let title = match manga_obj.get("title").as_string() {
					Ok(node) => node.read(),
					Err(_) => continue,
				};
				let mut id = match manga_obj.get("slug").as_string() {
					Ok(node) => node.read(),
					Err(_) => continue,
				};
				id += "|";

				id += &(match manga_obj.get("hid").as_string() {
					Ok(node) => node.read(),
					Err(_) => continue,
				});

				let cover = match manga_obj.get("cover_url").as_string() {
					Ok(node) => node.read(),
					Err(_) => continue,
				};
				if get_lang_code().unwrap_or_else(|| String::from("en")) == "zh-hk" {
					let manga_json = Request::new(
						&format!("{}/comic/{}?tachiyomi=true", api_url, id.clone()),
						HttpMethod::Get,
					)
					.json()?
					.as_object()?;
					let mut lang_list = Vec::new();
					for lang in manga_json.get("langList").as_array()? {
						lang_list.push(match lang.as_string() {
							Ok(node) => node.read(),
							Err(_) => continue,
						})
					}
					if !lang_list.contains(&get_lang_code().unwrap_or_else(|| String::from("en"))) {
						continue;
					}
				}
				mangas.push(Manga {
					id,
					cover,
					title,
					author: String::new(),
					artist: String::new(),
					description: String::new(),
					url: String::new(),
					categories: Vec::new(),
					status: MangaStatus::Unknown,
					nsfw: MangaContentRating::Safe,
					viewer: MangaViewer::Rtl,
				});
			}
		}
	}
	Ok(MangaPageResult {
		manga: mangas,
		has_more: true,
	})
}

pub fn parse_manga_details(api_url: String, id: String) -> Result<Manga> {
	let url = format!(
		"{}/comic/{}?tachiyomi=true",
		api_url,
		id.split('|').next().unwrap_or("")
	);
	let json = Request::new(url, HttpMethod::Get).json()?.as_object()?;
	let data = json.get("comic").as_object()?;
	let title = data_from_json(&data, "title");
	let description = unescape_html_entities(data_from_json(&data, "desc"));
	let status = manga_status(data.get("status").as_int().unwrap_or(0));
	let genres = data.get("md_comic_md_genres").as_array()?;
	let categories = genres
		.map(|genre| {
			let genre_obj = genre.as_object()?.get("md_genres").as_object()?;
			Ok(genre_obj.get("name").as_string()?.read())
		})
		.map(|a: Result<String>| a.unwrap_or_default())
		.collect::<Vec<String>>();
	let nsfw = if data.get("hentai").as_bool().unwrap_or(false) {
		MangaContentRating::Nsfw
	} else if data.get("content_rating").as_string()?.read() == "Suggestive" {
		MangaContentRating::Suggestive
	} else {
		MangaContentRating::Safe
	};

	let viewer = match data.get("country").as_string()?.read().as_str() {
		"kr" | "cn" => MangaViewer::Scroll,
		_ => MangaViewer::Rtl,
	};
	let cover = data_from_json(&data, "cover_url");
	let authors = json.get("authors").as_array()?;
	let author = if authors.is_empty() {
		String::from("")
	} else {
		authors.get(0).as_object()?.get("name").as_string()?.read()
	};
	let artists = json.get("artists").as_array()?;
	let artist = if artists.is_empty() {
		String::from("")
	} else {
		artists.get(0).as_object()?.get("name").as_string()?.read()
	};
	Ok(Manga {
		id: id.clone(),
		cover,
		title,
		author,
		artist,
		description,
		url: format!(
			"https://comick.app/comic/{}",
			id.split('|').next().unwrap_or("")
		),
		categories,
		status,
		nsfw,
		viewer,
	})
}

pub fn parse_chapter_list(api_url: String, id: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	let mut chapter_limit = 100;
	let page = 1;
	let cid = id.split('|').nth(1).unwrap_or("");
	let url = format!(
		"{}/comic/{}/chapters?limit={}&page={}&lang={}",
		api_url,
		cid,
		100,
		page,
		get_lang_code().unwrap_or_else(|| String::from("en"))
	);
	let json = Request::new(url, HttpMethod::Get).json()?.as_object()?;
	let total = json.get("total").as_int().unwrap_or(-1) as i32;
	if total != chapter_limit {
		chapter_limit = total;
	}
	let url = format!(
		"{}/comic/{}/chapters?limit={}&page={}&lang={}",
		api_url,
		cid,
		chapter_limit,
		page,
		get_lang_code().unwrap_or_else(|| String::from("en"))
	);
	let json = Request::new(url, HttpMethod::Get).json()?.as_object()?;
	let mchapters = json.get("chapters").as_array()?;
	for chapter in mchapters {
		let chapter_obj = chapter.as_object()?;
		let title = data_from_json(&chapter_obj, "title");
		let volume = chapter_obj.get("vol").as_float().unwrap_or(-1.0) as f32;
		let hid = data_from_json(&chapter_obj, "hid");
		let chapter = chapter_obj.get("chap").as_float().unwrap_or(0.0) as f32;
		let date_updated = chapter_obj
			.get("created_at")
			.as_date("yyyy-MM-dd'T'HH:mm:ssZ", Some("en_US"), None)
			.unwrap_or(-1.0);
		let scanlator = match chapter_obj.get("group_name").as_array() {
			Ok(str) => match str.get(0).as_string() {
				Ok(str) => str.read(),
				Err(_) => String::new(),
			},
			Err(_) => String::new(),
		};
		chapters.push(Chapter {
			id: hid,
			title,
			volume,
			chapter,
			date_updated,
			scanlator,
			url: String::new(),
			lang: get_lang_code().unwrap_or_else(|| String::from("en")),
		});
	}
	Ok(chapters)
}

pub fn parse_page_list(api_url: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	let url = format!("{}/chapter/{}?tachiyomi=true", api_url, chapter_id);
	let json = Request::new(url, HttpMethod::Get).json()?.as_object()?;
	if let Ok(chapter_obj) = json.get("chapter").as_object() {
		if let Ok(images) = chapter_obj.get("images").as_array() {
			let mut at = 0;
			for image in images {
				if let Ok(image_obj) = image.as_object() {
					let page_url = image_obj.get("url").as_string()?.read();
					pages.push(Page {
						index: at,
						url: page_url,
						base64: String::new(),
						text: String::new(),
					});
					at += 1;
				}
			}
		}
	}
	Ok(pages)
}

pub fn modify_image_request(base_url: String, request: Request) {
	request.header("Referer", &base_url);
}

pub fn handle_url(base_url: String, url: String) -> Result<DeepLink> {
	Ok(DeepLink {
		manga: Some(parse_manga_details(base_url, url)?),
		chapter: None,
	})
}
