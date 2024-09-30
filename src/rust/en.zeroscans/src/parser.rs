use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	prelude::format,
	std::{
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaPageResult, MangaStatus, Page,
};

use crate::helper::*;
use crate::BASE_URL;

pub fn parse_manga_list(filters: Vec<Filter>) -> Result<MangaPageResult> {
	let mut search_query = String::new();
	let mut include_genres = Vec::new();
	let mut exclude_genres = Vec::new();
	let mut status = "";

	let manga = all_comics()?;

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(value) = filter.value.as_string() {
					search_query = value.read().to_lowercase();
				}
			}
			FilterType::Genre => {
				if let Ok(name) = filter.object.get("name").as_string() {
					match filter.value.as_int().unwrap_or(-1) {
						0 => exclude_genres.push(name.read()),
						1 => include_genres.push(name.read()),
						_ => continue,
					}
				}
			}
			FilterType::Select => match filter.name.as_str() {
				"Status" => match filter.value.as_int().unwrap_or(-1) {
					0 => status = "All",
					1 => status = "Completed",
					2 => status = "Dropped",
					3 => status = "Ongoing",
					4 => status = "Hiatus",
					_ => continue,
				},
				_ => continue,
			},
			_ => continue,
		}
	}

	let mut filtered_manga = manga.clone();

	if !include_genres.is_empty() {
		filtered_manga.retain(|manga| {
			include_genres
				.iter()
				.any(|genre| manga.categories.contains(genre))
		})
	}

	if !exclude_genres.is_empty() {
		filtered_manga.retain(|manga| {
			!exclude_genres
				.iter()
				.any(|genre| manga.categories.contains(genre))
		})
	}

	if !status.is_empty() {
		filtered_manga.retain(|manga| match status {
			"All" => true,
			"Completed" => manga.status == MangaStatus::Completed,
			"Dropped" => manga.status == MangaStatus::Cancelled,
			"Ongoing" => manga.status == MangaStatus::Ongoing,
			"Hiatus" => manga.status == MangaStatus::Hiatus,
			_ => false,
		})
	}

	if !search_query.is_empty() {
		filtered_manga.retain(|manga| manga.title.to_lowercase().contains(&search_query))
	}

	Ok(MangaPageResult {
		manga: filtered_manga,
		has_more: false,
	})
}

// The only alternative listing Zero Scans has is new chapters
pub fn parse_manga_listing() -> Result<MangaPageResult> {
	let url = format!("{}/swordflake/new-chapters", BASE_URL);

	let json = Request::new(url, HttpMethod::Get).json()?.as_object()?;
	let comics = json.get("all").as_array()?;

	let mut manga: Vec<Manga> = Vec::new();

	for comic in comics {
		let comic = comic.as_object()?;

		let title = comic.get("name").as_string()?.read();
		let slug = comic.get("slug").as_string()?.read();
		let id = comic.get("id").as_int()?;
		let url = format!("{}/comics/{}", BASE_URL, slug);
		let cover = comic
			.get("cover")
			.as_object()?
			.get("vertical")
			.as_string()?
			.read();

		manga.push(Manga {
			id: format!("[<{}>]{}", id, slug),
			cover,
			title,
			url,
			..Default::default()
		})
	}

	Ok(MangaPageResult {
		manga,
		has_more: false,
	})
}

pub fn parse_manga_details(manga_id: String) -> Result<Manga> {
	let manga = all_comics()?
		.into_iter()
		.find(|manga| manga.id == manga_id)
		.ok_or(AidokuError {
			reason: AidokuErrorKind::Unimplemented,
		})?;

	Ok(manga)
}

pub fn parse_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let mut all_chapters: Vec<Chapter> = Vec::new();
	let mut page = 1;

	let (id, slug) = get_identifiers(&manga_id)?;

	loop {
		let url = format!(
			"{}/swordflake/comic/{}/chapters?sort=desc&page={}",
			BASE_URL, id, &page
		);

		let json = Request::new(url, HttpMethod::Get).json()?.as_object()?;
		let data = json.get("data").as_object()?;

		let chapters = data.get("data").as_array()?;
		let last_page = data.get("last_page").as_int()?;

		if !chapters.is_empty() {
			for chapter in chapters {
				let chapter = chapter.as_object()?;

				let id = chapter.get("id").as_int()?;
				let chapter_number = chapter.get("name").as_float()? as f32;
				let date_updated = chapter.get("created_at").as_string()?.read();
				let date_updated = get_date(date_updated);
				let scanlator = chapter.get("group").as_string().unwrap_or("".into()).read();
				let chapter_url = format!("{}/comics/{}/{}", BASE_URL, slug, id);

				all_chapters.push(Chapter {
					id: format!("{}", id),
					chapter: chapter_number,
					date_updated,
					scanlator,
					url: chapter_url,
					..Default::default()
				});
			}
		}

		if page == last_page {
			break;
		} else {
			page += 1;
		}
	}

	Ok(all_chapters)
}

pub fn parse_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let (_, slug) = get_identifiers(&manga_id)?;

	let url = format!(
		"{}/swordflake/comic/{}/chapters/{}",
		BASE_URL, slug, chapter_id
	);

	let json = Request::new(url, HttpMethod::Get).json()?.as_object()?;
	let data = json.get("data").as_object()?;

	let chapter = data.get("chapter").as_object()?;
	let pages = chapter.get("high_quality").as_array()?;

	let mut page_list: Vec<Page> = Vec::new();

	for (index, page) in pages.enumerate() {
		let page = page.as_string()?.read();

		page_list.push(Page {
			index: index as i32,
			url: page,
			..Default::default()
		});
	}

	Ok(page_list)
}

pub fn modify_image_request(request: Request) {
	request.header("Referer", BASE_URL);
}

pub fn handle_url(url: String) -> Result<DeepLink> {
	if let Some((slug, chapter_id)) = parse_url(&url) {
		let data = all_comics()?;
		let comic = data.into_iter().find(|comic| comic.id.contains(&slug));

		if let Some(chapter_id) = chapter_id {
			Ok(DeepLink {
				manga: comic,
				chapter: Some(Chapter {
					id: chapter_id,
					..Default::default()
				}),
			})
		} else {
			Ok(DeepLink {
				manga: comic,
				..Default::default()
			})
		}
	} else {
		Err(AidokuError {
			reason: AidokuErrorKind::Unimplemented,
		})
	}
}
