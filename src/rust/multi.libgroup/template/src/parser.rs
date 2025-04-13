use aidoku::error::Result;
use aidoku::prelude::format;
use aidoku::std::defaults::defaults_get;
use aidoku::std::String;
use aidoku::{std::ObjectRef, Manga, MangaPageResult};
use aidoku::{Chapter, MangaContentRating, MangaViewer, Page};
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::helpers::{display_title, extract_f32_from_string, id_to_status};
use crate::template::CDN;
extern crate alloc;

pub fn parse_manga_list(
	js: ObjectRef,
	domain: &String,
	nsfw: &MangaContentRating,
) -> Result<MangaPageResult> {
	let has_more = js.get("meta").as_object()?.get("has_next_page").as_bool()?;
	let mangas = js.get("data").as_array()?;
	let mut manga: Vec<Manga> = Vec::new();

	for data in mangas {
		if let Ok(data_obj) = data.as_object() {
			let title = match data_obj.get(&display_title()).as_string() {
				Ok(x) => x.read(),
				Err(_) => continue,
			};

			let id = match data_obj.get("slug_url").as_string() {
				Ok(x) => x.to_string(),
				Err(_) => continue,
			};

			let cover = match data_obj
				.get("cover")
				.as_object()?
				.get("default")
				.as_string()
			{
				Ok(x) => x.read(),
				Err(_) => continue,
			};

			let url = match data_obj.get("slug_url").as_string() {
				Ok(x) => format!("https://{}/ru/manga/{}", domain, x.read()),
				Err(_) => continue,
			};

			let status = match data_obj.get("status").as_object()?.get("id").as_int() {
				Ok(x) => id_to_status(x),
				Err(_) => continue,
			};

			let viewer = aidoku::MangaViewer::Rtl;

			manga.push(Manga {
				id,
				cover,
				title,
				url,
				status,
				nsfw: *nsfw,
				viewer,
				..Default::default()
			})
		}
	}

	Ok(MangaPageResult { manga, has_more })
}

pub fn parse_manga_details(
	js: ObjectRef,
	domain: &str,
	is_nsfw: &MangaContentRating,
) -> Result<Manga> {
	let detail = js.get("data").as_object()?;

	let id = detail.get("slug_url").as_string()?.read();

	let cover = detail
		.get("cover")
		.as_object()?
		.get("default")
		.as_string()?
		.read();

	let title = detail.get(&display_title()).as_string()?.read();

	let authors = detail.get("authors").as_array()?;
	let author = authors
		.map(|author| {
			let author_object = author.as_object()?;
			Ok(author_object.get("name").as_string()?.read())
		})
		.map(|a: Result<String>| a.unwrap_or_default())
		.collect::<Vec<String>>()
		.join(", ");

	let artists = detail.get("artists").as_array()?;
	let artist = artists
		.map(|artist| {
			let artist_object = artist.as_object()?;
			Ok(artist_object.get("name").as_string()?.read())
		})
		.map(|x: Result<String>| x.unwrap_or_default())
		.collect::<Vec<String>>()
		.join(", ");

	let description = detail.get("summary").as_string()?.read();

	let url = format!(
		"https://{}/ru/manga/{}",
		domain,
		detail.get("slug_url").as_string()?.read()
	);

	let categories: Vec<String> = detail
		.get("genres")
		.as_array()?
		.map(|genre| {
			let genre_object = genre.as_object()?;
			Ok(genre_object.get("name").as_string()?.read())
		})
		.map(|x: Result<String>| x.unwrap_or_default())
		.collect::<Vec<String>>();

	let status = id_to_status(
		detail
			.get("status")
			.as_object()?
			.get("id")
			.as_int()
			.unwrap_or_default(),
	);

	let nsfw = *is_nsfw;

	let viewer = if detail
		.get("type")
		.as_object()?
		.get("id")
		.as_int()
		.unwrap_or_default()
		== 5
	{
		MangaViewer::Scroll
	} else {
		MangaViewer::Rtl
	};

	Ok(Manga {
		id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories,
		status,
		nsfw,
		viewer,
	})
}

pub fn parse_chapter_list(js: ObjectRef, id: &str, domain: &str) -> Result<Vec<Chapter>> {
	let chapters: Vec<Chapter> = js
		.get("data")
		.as_array()?
		.map(|chapter| {
			let chapter_object = chapter.as_object()?;

			// chapter_id: 1#1
			// Scheme: number#volume

			Ok(Chapter {
				id: format!(
					"{}#{}",
					extract_f32_from_string(
						String::new(),
						chapter_object.get("number").as_string()?.read()
					)[0],
					extract_f32_from_string(
						String::new(),
						chapter_object.get("volume").as_string()?.read()
					)[0]
				),

				title: chapter_object
					.get("name")
					.as_string()
					.unwrap_or("".into())
					.to_string(),
				volume: extract_f32_from_string(
					String::new(),
					chapter_object.get("volume").as_string()?.read(),
				)[0],
				chapter: extract_f32_from_string(
					String::new(),
					chapter_object.get("number").as_string()?.read(),
				)[0],
				date_updated: chapter_object
					.get("branches")
					.as_array()?
					.get(0)
					.as_object()?
					.get("created_at")
					.as_string()?
					.as_date("yyyy-MM-dd'T'HH:mm:ss.SSS'Z", Some("en_US"), None),
				scanlator: chapter_object
					.get("branches")
					.as_array()?
					.get(0)
					.as_object()?
					.get("user")
					.as_object()?
					.get("username")
					.as_string()?
					.read(),
				url: format!("https://{}/{}", domain, id),
				lang: "ru".to_string(),
			})
		})
		.map(|x: Result<Chapter>| x.unwrap())
		.rev()
		.collect::<Vec<Chapter>>();

	Ok(chapters)
}

pub fn parse_image_servers_list(js: ObjectRef, site_id: i64) -> Result<CDN> {
	let (mut main, mut second, mut compress) = (String::new(), String::new(), String::new());
	let image_servers_list = js.get("data").as_object()?.get("imageServers").as_array()?;

	for image_server in image_servers_list {
		let image_server_obj = image_server.as_object()?;

		if image_server_obj
			.get("site_ids")
			.as_array()?
			.any(|i| i.as_int().unwrap() == site_id)
		{
			let url = image_server_obj.get("url").as_string()?.read();
			let id = image_server_obj.get("id").as_string()?.read();

			match id.as_str() {
				"main" => main = url.clone(),
				"secondary" => second = url.clone(),
				"compress" => compress = url.clone(),
				_ => {}
			}
		}
	}

	Ok(CDN {
		main,
		second,
		compress,
	})
}

pub fn parse_page_list(js: ObjectRef, cdn: &CDN) -> Result<Vec<Page>> {
	let chapters: Vec<Page> = js
		.get("data")
		.as_object()?
		.get("pages")
		.as_array()?
		.map(|page| {
			let page_object = page.as_object()?;
			let image_server = match defaults_get("server_image")
				.unwrap()
				.as_string()
				.unwrap()
				.read()
				.as_str()
			{
				"main" => &cdn.main,
				"second" => &cdn.second,
				"compression" => &cdn.compress,
				_ => &cdn.compress,
			};

			let url = format!(
				"{}{}",
				image_server,
				page_object.get("url").as_string()?.read()
			);
			let index = page_object.get("slug").as_int().unwrap() as i32;

			Ok(Page {
				index,
				url,
				..Default::default()
			})
		})
		.map(|x: Result<Page>| x.unwrap())
		.collect::<Vec<Page>>();

	Ok(chapters)
}
