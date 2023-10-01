#![no_std]
extern crate alloc;
mod decryptor;
mod parser;
mod url;

use aidoku::{
	error::Result,
	helpers::substring::Substring,
	prelude::{get_chapter_list, get_manga_details, get_manga_list, get_page_list, handle_url},
	std::{String, Vec},
	Chapter, DeepLink, Filter, Manga, MangaPageResult, MangaStatus, Page,
};
use alloc::string::ToString;
use decryptor::EncryptedString;
use parser::{Element, JsonObj, JsonString, MangaListResponse, NodeArrValue, UuidString};
use url::{Url, CHAPTER_PATH, MANGA_PATH};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let manga_list_url = Url::from((filters, page));

	if let Url::Filters { .. } = manga_list_url {
		let filters_page = manga_list_url.get_html()?;
		return filters_page.get_page_result();
	}

	let search_json = manga_list_url.get_json()?;
	search_json.get_page_result()
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let manga_page = Url::Manga(&manga_id).get_html()?;

	let cover = manga_page
		.get_attr("img.lazyload", "data-src")
		.replace(".328x422.jpg", "");

	let title = manga_page.get_text("h6");

	let artist = manga_page
		.select("span.comicParticulars-right-txt > a")
		.array()
		.filter_map(NodeArrValue::ok_text)
		.collect::<Vec<_>>()
		.join("、");

	let description = manga_page.get_text("p.intro");

	let manga_url = Url::Manga(&manga_id).to_string();

	let categories = manga_page
		.select("span.comicParticulars-left-theme-all.comicParticulars-tag > a")
		.array()
		.filter_map(NodeArrValue::ok_text)
		.map(|str| str[1..].to_string())
		.collect::<Vec<_>>();

	let status_str = manga_page.get_text("li:contains(狀態：) > span.comicParticulars-right-txt");
	let status = match status_str.as_str() {
		"連載中" => MangaStatus::Ongoing,
		"已完結" | "短篇" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};

	Ok(Manga {
		id: manga_id,
		cover,
		title,
		author: artist.clone(),
		artist,
		description,
		url: manga_url,
		categories,
		status,
		..Default::default()
	})
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let mut chapters = Vec::<Chapter>::new();

	let mut chapter_vec = Vec::<(String, String, f64)>::new();
	let groups_values = Url::ChapterList(&manga_id)
		.get_json()?
		.as_object()?
		.get_as_string("results")?
		.decrypt()
		.json()?
		.as_object()?
		.get("groups")
		.as_object()?
		.values();
	for groups_value in groups_values {
		let chapters_arr = groups_value.as_object()?.get("chapters").as_array()?;
		for chapters_value in chapters_arr {
			let chapters_obj = chapters_value.as_object()?;

			let id = chapters_obj.get_as_string("id")?;
			let name = chapters_obj.get_as_string("name")?;
			let timestamp = id.get_timestamp();

			chapter_vec.push((id, name, timestamp));
		}
	}
	chapter_vec.sort_by(|a, b| a.2.total_cmp(&b.2));

	for (index, (chapter_id, title, date_updated)) in chapter_vec.iter().enumerate() {
		let chapter_num = (index + 1) as f32;

		let chapter_url = Url::Chapter(&manga_id, chapter_id).to_string();

		let chapter = Chapter {
			id: chapter_id.clone(),
			title: title.clone(),
			chapter: chapter_num,
			date_updated: *date_updated,
			url: chapter_url,
			lang: "zh".to_string(),
			..Default::default()
		};
		chapters.insert(0, chapter);
	}

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut pages = Vec::<Page>::new();

	let page_arr = Url::Chapter(&manga_id, &chapter_id)
		.get_html()?
		.get_attr("div.imageData", "contentkey")
		.decrypt()
		.json()?
		.as_array()?;

	for (index, page_value) in page_arr.enumerate() {
		let page_url = page_value.as_object()?.get_as_string("url")?;

		pages.push(Page {
			index: index as i32,
			url: page_url,
			..Default::default()
		});
	}

	Ok(pages)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let Some(path) = url.substring_after(MANGA_PATH) else {
		return Ok(DeepLink::default());
	};

	let Some(chapter_id) = path.substring_after(CHAPTER_PATH) else {
		let manga = get_manga_details(path.to_string())?;

		return Ok(DeepLink {
			manga: Some(manga),
			chapter: None,
		});
	};
	let chapter = Chapter {
		id: chapter_id.to_string(),
		..Default::default()
	};

	let Some(manga_id) = path.substring_before(CHAPTER_PATH) else {
		return Ok(DeepLink {
			manga: None,
			chapter: Some(chapter),
		});
	};
	let manga = get_manga_details(manga_id.to_string())?;

	Ok(DeepLink {
		manga: Some(manga),
		chapter: Some(chapter),
	})
}
