//
// source made by apix <@apix0n>
//

#![no_std]
use aidoku::{
	error::Result,
	helpers::uri::encode_uri,
	prelude::*,
	std::net::Request,
	std::{String, Vec},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult,
	MangaStatus, MangaViewer, Page,
};

const BASE_URL: &str = "https://bluesolo.org";
const API_BASE_URL: &str = "https://bluesolo.org/api";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, _page: i32) -> Result<MangaPageResult> {
	let mut url = String::new();

	// In order: search -> target -> genre
	for filter in filters {
		if url.is_empty() {
			match filter.kind {
				FilterType::Title => {
					if let Ok(query) = filter.value.as_string() {
						url = format!("{}/search/{}", API_BASE_URL, encode_uri(query.read()));
					}
				}
				FilterType::Select => {
					let index = filter.value.as_int().unwrap_or(0);
					if index > 0 {
						// Skip "Tous" option (index 0)
						match filter.name.as_str() {
							"Public" => {
								let target = match index {
									1 => "shonen",
									2 => "seinen",
									_ => continue,
								};
								url = format!("{}/targets/{}", API_BASE_URL, target);
							}
							"Genres" => {
								let genre = match index {
									1 => "action",
									2 => "aliens",
									3 => "aventure",
									4 => "comedy",
									5 => "crime",
									6 => "dark-fantasy",
									7 => "drama",
									8 => "ecole",
									9 => "fantasy",
									10 => "fantome",
									11 => "gore",
									12 => "horreur",
									13 => "mafia",
									14 => "magie",
									15 => "mature",
									16 => "one-shot",
									17 => "psychologique",
									18 => "romance",
									19 => "sci-fi",
									20 => "sport",
									21 => "surnaturel",
									22 => "survival",
									23 => "thriller",
									24 => "tranche-de-vie",
									25 => "western",
									_ => continue,
								};
								url = format!("{}/genres/{}", API_BASE_URL, genre);
							}
							_ => continue,
						}
					}
				}
				_ => continue,
			}
		}
	}

	// If no filter was selected, use default comics endpoint
	if url.is_empty() {
		url = format!("{}/comics", API_BASE_URL);
	}

	let json = Request::get(&url).json()?;

	let comics = json
		.as_object()?
		.get("comics")
		.as_array()
		.unwrap_or_default();

	let mut manga = Vec::new();
	for comic in comics {
		let obj = comic.as_object()?;
		if let Ok(id) = obj.get("slug").as_string().map(|s| s.read()) {
			if let (Ok(title), Ok(cover)) = (
				obj.get("title").as_string().map(|s| s.read()),
				obj.get("thumbnail").as_string().map(|s| s.read()),
			) {
				let nsfw = obj
					.get("adult")
					.as_int()
					.map(|a| {
						if a == 1 {
							MangaContentRating::Nsfw
						} else {
							MangaContentRating::Safe
						}
					})
					.unwrap_or(MangaContentRating::Safe);

				manga.push(Manga {
					id,
					title,
					cover,
					nsfw,
					..Default::default()
				});
			}
		}
	}

	Ok(MangaPageResult {
		manga,
		has_more: false,
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, _page: i32) -> Result<MangaPageResult> {
	match listing.name.as_str() {
		"Recommandé" => {
			let url = format!("{}/recommended", API_BASE_URL);
			let json = Request::get(&url).json()?;

			let comics = json
				.as_object()?
				.get("comics")
				.as_array()
				.unwrap_or_default();

			let mut manga = Vec::new();
			for comic in comics {
				let obj = comic.as_object()?;
				if let Ok(id) = obj.get("slug").as_string().map(|s| s.read()) {
					if let (Ok(title), Ok(cover)) = (
						obj.get("title").as_string().map(|s| s.read()),
						obj.get("thumbnail").as_string().map(|s| s.read()),
					) {
						let nsfw = obj
							.get("adult")
							.as_int()
							.map(|a| {
								if a == 1 {
									MangaContentRating::Nsfw
								} else {
									MangaContentRating::Safe
								}
							})
							.unwrap_or(MangaContentRating::Safe);

						manga.push(Manga {
							id,
							title,
							cover,
							nsfw,
							..Default::default()
						});
					}
				}
			}

			Ok(MangaPageResult {
				manga,
				has_more: false,
			})
		}
		_ => Ok(MangaPageResult {
			manga: Vec::new(),
			has_more: false,
		}),
	}
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{}/comics/{}", API_BASE_URL, id);
	let json = Request::get(&url).json()?;

	let root = json.as_object()?;
	let comic = root.get("comic").as_object()?;

	let slug = comic.get("slug").as_string()?.read();
	let title = comic.get("title").as_string()?.read();
	let cover = comic.get("thumbnail").as_string()?.read();

	// Extract genres/categories from "genres" array
	let categories = comic
		.get("genres")
		.as_array()
		.map(|arr| {
			let mut genres = Vec::new();
			for genre in arr {
				if let Ok(genre_obj) = genre.as_object() {
					if let Ok(name) = genre_obj.get("name").as_string() {
						genres.push(name.read());
					}
				}
			}
			genres
		})
		.unwrap_or_default();

	let nsfw = comic
		.get("adult")
		.as_int()
		.map(|a| {
			if a == 1 {
				MangaContentRating::Nsfw
			} else {
				MangaContentRating::Safe
			}
		})
		.unwrap_or(MangaContentRating::Safe);

	Ok(Manga {
		id: slug.clone(),
		title,
		cover,
		author: comic
			.get("author")
			.as_string()
			.map(|s| s.read())
			.unwrap_or_default(),
		artist: comic
			.get("artist")
			.as_string()
			.map(|s| s.read())
			.unwrap_or_default(),
		url: format!("{}/comics/{}", BASE_URL, slug),
		categories,
		description: comic
			.get("description")
			.as_string()
			.map(|s| s.read())
			.unwrap_or_default(),
		status: comic
			.get("status")
			.as_string()
			.map(|s| match s.read().as_str() {
				"En cours" => MangaStatus::Ongoing,
				"Terminé" => MangaStatus::Completed,
				"En hiatus" => MangaStatus::Hiatus,
				"Annulé" => MangaStatus::Cancelled,
				_ => MangaStatus::Unknown,
			})
			.unwrap_or(MangaStatus::Unknown),
		nsfw,
		viewer: MangaViewer::Rtl,
	})
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/comics/{}", API_BASE_URL, manga_id);
	let json = Request::get(&url).json()?;

	let root = json.as_object()?;
	let comic = root.get("comic").as_object()?;
	let chapters = comic.get("chapters").as_array()?;

	let mut chapter_list = Vec::new();

	for chapter in chapters {
		let obj = chapter.as_object()?;

		// handle chapter and subchapter numbers
		let chapter_num = obj.get("chapter").as_float().unwrap_or_default() as f32;
		let subchapter = match obj.get("subchapter").as_float() {
			Ok(num) => Some(num as f32),
			_ => None,
		};

		let id = obj.get("slug_lang_vol_ch_sub").as_string()?.read();
		let title = obj
			.get("title")
			.as_string()
			.map(|s| s.read())
			.unwrap_or_default();

		let volume = obj
			.get("volume")
			.as_float()
			.map(|v| v as f32)
			.unwrap_or(-1.0);

		let scanlator = obj
			.get("teams")
			.as_array()
			.map(|teams| {
				let mut names = Vec::new();
				for team in teams {
					if let Ok(team_obj) = team.as_object() {
						if let Ok(name) = team_obj.get("name").as_string() {
							names.push(name.read());
						}
					}
				}
				names.join(", ")
			})
			.unwrap_or_default();

		// url constructor for chapters, first check is for subchapters, other one for
		// regular chapters
		let url = if let Some(sub) = subchapter {
			format!(
				"{}/read/{}/fr/ch/{}/sub/{}",
				BASE_URL, manga_id, chapter_num as i32, sub as i32
			)
		} else {
			format!(
				"{}/read/{}/fr/ch/{}",
				BASE_URL, manga_id, chapter_num as i32
			)
		};

		// adjusts chapter number if there's a subchapter
		let chapter_num = if let Some(sub) = subchapter {
			chapter_num + (sub / 10.0)
		} else {
			chapter_num
		};

		chapter_list.push(Chapter {
			id: id.clone(),
			title,
			volume,
			chapter: chapter_num,
			date_updated: obj
				.get("published_on")
				.as_date("yyyy-MM-dd'T'HH:mm:ss.SSSSSSZ", Some("fr"), None)
				.unwrap_or_default(),
			scanlator,
			url,
			lang: String::from("fr"),
		});
	}

	Ok(chapter_list)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let parts: Vec<&str> = chapter_id.split('-').collect();
	if parts.len() != 4 {
		return Ok(Vec::new());
	}

	let chapter_num = parts[2];
	let sub_part = parts[3];

	// url constructor for chapters, first check is for subchapters, other one for
	// regular chapters
	let url = if sub_part == "N" {
		format!("{}/read/{}/fr/ch/{}", API_BASE_URL, manga_id, chapter_num)
	} else {
		format!(
			"{}/read/{}/fr/ch/{}/sub/{}",
			API_BASE_URL, manga_id, chapter_num, sub_part
		)
	};

	let json = Request::get(&url).json()?;
	let root = json.as_object()?;
	let chapter = root.get("chapter").as_object()?;
	let pages = chapter.get("pages").as_array()?;

	let mut page_list = Vec::new();

	for (index, url_value) in pages.enumerate() {
		if let Ok(url) = url_value.as_string().map(|s| s.read()) {
			page_list.push(Page {
				index: index as i32,
				url,
				..Default::default()
			});
		}
	}

	Ok(page_list)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.header("Referer", BASE_URL);
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	// Remove base URL and split the path
	if !url.starts_with(BASE_URL) {
		return Ok(DeepLink::default());
	}

	let path = url.replace(BASE_URL, "");
	let parts: Vec<&str> = path.split('/').collect();

	if parts.len() < 3 {
		return Ok(DeepLink::default());
	}

	// Handle for manga detail URLs: /comics/{manga_id}
	if parts[1] == "comics" {
		// Get the full manga ID by joining all remaining parts with '/'
		let manga_id = parts[2..].join("/");

		let manga = Manga {
			id: manga_id.clone(),
			url: url.clone(), // Use the full URL as the manga URL,
			..Default::default()
		};

		return Ok(DeepLink {
			manga: Some(manga),
			chapter: None,
		});
	}

	// Handle for chapter URLs:
	// /read/{manga_id}/fr/ch/{chapter_number}(/sub/{subchapter})
	if parts[1] == "read" && parts.len() >= 6 {
		let manga_id = String::from(parts[2]);
		let chapter_num = parts[5].parse::<f32>().unwrap_or_default();

		// check if there's a subchapter and adjust the chapter number accordingly
		let chapter_number = if parts.len() >= 8 && parts[6] == "sub" {
			let sub = parts[7].parse::<f32>().unwrap_or_default();
			chapter_num + (sub / 10.0)
		} else {
			chapter_num
		};

		// creates the Chapter object
		let chapter = Chapter {
			id: format!("{}-{}", manga_id, chapter_number),
			volume: -1.0,
			chapter: chapter_number,
			url: url.clone(), // Use the full URL as the chapter URL
			lang: String::from("fr"),
			..Default::default()
		};

		// creates a basic Manga object
		let manga = Manga {
			id: manga_id.clone(),
			url: format!("{}/comics/{}", BASE_URL, manga_id), // Construct proper manga URL
			..Default::default()
		};

		return Ok(DeepLink {
			manga: Some(manga),
			chapter: Some(chapter),
		});
	}

	Ok(DeepLink::default())
}
