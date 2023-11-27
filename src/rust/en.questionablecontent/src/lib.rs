#![no_std]
mod helper;
extern crate alloc;

use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	helpers::substring::*,
	prelude::*,
	std::{
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, DeepLink, Filter, Manga, MangaPageResult, Page,
};
use alloc::{collections::BTreeSet, string::ToString, vec};
use helper::*;

#[get_manga_list]
fn get_manga_list(_: Vec<Filter>, _: i32) -> Result<MangaPageResult> {
	Ok(MangaPageResult {
		manga: vec![comic_info()],
		has_more: false,
	})
}

#[get_manga_details]
fn get_manga_details(_: String) -> Result<Manga> {
	Ok(comic_info())
}

#[get_chapter_list]
fn get_chapter_list(_: String) -> Result<Vec<Chapter>> {
	let mut known_chapters = BTreeSet::new();
	let html = Request::new(
		"https://www.questionablecontent.net/archive.php",
		HttpMethod::Get,
	)
	.html()?;
	let mut res = html
		.select("#container > div.row > div.column > a")
		.array()
		.rev()
		.filter_map(|element| match element.as_node() {
			Ok(node) => parse_chapter_and_title(node.text().read()).and_then(|(chapter, title)| {
				if known_chapters.insert(chapter as i32) {
					Some(Chapter {
						id: chapter.to_string(),
						title,
						chapter,
						url: node.attr("abs:href").read(),
						..Default::default()
					})
				} else {
					None
				}
			}),
			Err(_) => None,
		})
		.rev()
		.collect::<Vec<_>>();

	// QC's archive feed is missing a few here and there..
	for n in 1..res.first().unwrap().chapter as i32 {
		if known_chapters.insert(n) {
			res.insert(
				n as usize,
				Chapter {
					id: n.to_string(),
					title: String::from("(no title available)"),
					chapter: n as f32,
					url: format!("https://www.questionablecontent.net/view.php?comic={n}"),
					..Default::default()
				},
			)
		}
	}
	Ok(res)
}

#[get_page_list]
fn get_page_list(_: String, id: String) -> Result<Vec<Page>> {
	let html = Request::new(
		format!("https://www.questionablecontent.net/view.php?comic={id}"),
		HttpMethod::Get,
	)
	.html()?;
	let strip_node = html.select("#strip");
	let blip_node = html.select("#newspost");

	let url = strip_node.attr("abs:src").read();
	let text = blip_node.text().read();

	Ok(vec![
		Page {
			index: 0,
			url,
			..Default::default()
		},
		Page {
			index: 1,
			url: newsblip_image_url(text),
			..Default::default()
		},
	])
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let url_parts = url.split('/').filter(|p| !p.is_empty()).collect::<Vec<_>>();

	let manga = match url_parts[1] {
		"www.questionablecontent.net" | "questionablecontent.net" => Some(comic_info()),
		_ => {
			return Err(AidokuError {
				reason: AidokuErrorKind::Unimplemented,
			})
		}
	};

	let chapter = match url.substring_after_last("?comic=") {
		Some(comic_raw) => match comic_raw.parse::<f32>() {
			Ok(chapter) => Some(Chapter {
				id: chapter.to_string(),
				chapter,
				url,
				..Default::default()
			}),
			Err(_) => None,
		},
		None => None,
	};

	Ok(DeepLink { manga, chapter })
}
