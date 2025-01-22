#![no_std]
extern crate alloc;
mod decryptor;
mod helper;
mod parser;
mod url;

use aidoku::{
	error::Result,
	prelude::{
		format, get_chapter_list, get_manga_details, get_manga_list, get_page_list, handle_url,
	},
	std::{defaults::defaults_get, String, Vec},
	Chapter, DeepLink, Filter, Manga, MangaPageResult, MangaStatus, Page,
};
use alloc::string::ToString;
use decryptor::EncryptedString;
use parser::{Element, JsonObj, JsonString, MangaListResponse, NodeArrValue, Part, UuidString};
use url::Url;
use uuid::Uuid;

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
	let manga_page = Url::Manga { id: &manga_id }.get_html()?;

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

	let manga_url = Url::Manga { id: &manga_id }.to_string();

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
	let group_values = Url::ChapterList { id: &manga_id }
		.get_json()?
		.as_object()?
		.get_as_string("results")?
		.decrypt()?
		.json()?
		.as_object()?
		.get("groups")
		.as_object()?
		.values();
	let groups = group_values
		.map(|group_value| {
			let group_obj = group_value.as_object()?;

			let group_name = group_obj.get_as_string("name")?;
			let title_prefix = if group_name == "默認" {
				String::new()
			} else {
				format!("{}：", group_name)
			};

			group_obj
				.get("chapters")
				.as_array()?
				.map(|chapter_value| {
					let chapter_obj = chapter_value.as_object()?;

					let chapter_id = chapter_obj.get_as_string("id")?;

					let chapter_name = chapter_obj.get_as_string("name")?;
					let title = format!("{}{}", title_prefix, chapter_name);

					let timestamp = chapter_id.get_timestamp();

					Ok((chapter_id, title, timestamp))
				})
				.collect::<Result<Vec<_>>>()
		})
		.collect::<Result<Vec<_>>>()?;

	let mut groups_iter = groups.iter();
	let mut sorted_chapters = groups_iter.next().cloned().unwrap_or_default();
	for unsorted_chapters in groups_iter {
		let mut index = 0;
		for unsorted_chapter in unsorted_chapters {
			while index < sorted_chapters.len() && unsorted_chapter.2? > sorted_chapters[index].2? {
				index += 1;
			}
			sorted_chapters.insert(index, unsorted_chapter.clone());
			index += 1;
		}
	}

	let chapters = sorted_chapters
		.iter()
		.map(|(chapter_id, title, res_date_updated)| {
			let part = title.parse::<Part>()?;

			let date_updated = (*res_date_updated)?;

			let chapter_url = Url::Chapter {
				manga_id: &manga_id,
				chapter_id,
			}
			.to_string();

			Ok(Chapter {
				id: chapter_id.clone(),
				title: part.title,
				volume: part.volume,
				chapter: part.chapter,
				date_updated,
				url: chapter_url,
				lang: "zh".to_string(),
				..Default::default()
			})
		})
		.rev()
		.collect::<Result<_>>()?;

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut pages = Vec::<Page>::new();

	let page_arr = Url::Chapter {
		manga_id: &manga_id,
		chapter_id: &chapter_id,
	}
	.get_html()?
	.get_attr("div.imageData", "contentkey")
	.decrypt()?
	.json()?
	.as_array()?;

	let image_format = defaults_get("imageFormat").and_then(|v| v.as_string().map(|v| v.read()))?;

	let image_quality =
		defaults_get("imageQuality").and_then(|v| v.as_string().map(|v| v.read()))?;

	let image_ext = format!("{}.{}", image_quality, image_format);

	for (index, page_value) in page_arr.enumerate() {
		let page_url = page_value
			.as_object()?
			.get_as_string("url")?
			.replace("c800x.jpg", &image_ext);

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
	let parts = url.split('/').skip(3).collect::<Vec<_>>();
	let (manga_id, chapter_id) = match parts[..] {
		["comic", manga_id] | ["h5", "details", "comic", manga_id] => (manga_id, None),

		["comic", manga_id, "chapter", chapter_id]
		| ["h5", "comicContent", manga_id, chapter_id] => (manga_id, Some(chapter_id)),

		_ => return Ok(DeepLink::default()),
	};

	if !manga_id
		.chars()
		.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
	{
		return Ok(DeepLink::default());
	}

	let manga = get_manga_details(manga_id.into())?;

	let chapter = chapter_id
		.filter(|id| id.parse::<Uuid>().is_ok())
		.map(|id| Chapter {
			id: id.into(),
			..Default::default()
		});

	Ok(DeepLink {
		manga: Some(manga),
		chapter,
	})
}
