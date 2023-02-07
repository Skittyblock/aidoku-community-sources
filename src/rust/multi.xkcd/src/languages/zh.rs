use super::THUMBNAIL_URL;
use aidoku::{
	error::Result,
	prelude::format,
	std::{
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, Manga, MangaContentRating, MangaStatus, MangaViewer, Page,
};
use alloc::string::ToString;

pub fn comic_info() -> Manga {
	Manga {
		id: String::from("multi.xkcd.zh"),
		cover: String::from(THUMBNAIL_URL),
		title: String::from("xkcd 中文翻譯"),
		author: String::from("兰德尔·门罗"),
		artist: String::from("兰德尔·门罗"),
		description: String::from("這裡翻譯某個關於浪漫、諷刺、數學、以及語言的漫畫"),
		url: String::from("https://xkcd.tw"),
		categories: Vec::new(),
		status: MangaStatus::Ongoing,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Ltr,
	}
}

pub fn get_chapter_list() -> Result<Vec<Chapter>> {
	let json = Request::new("https://xkcd.tw/api/strips.json", HttpMethod::Get)
		.json()?
		.as_object()?;
	Ok(json
		.values()
		.map(|strip| {
			let stripobj = strip.as_object()?;
			let chapter = stripobj.get("id").as_int().unwrap_or(-1);
			let date_updated = stripobj
				.get("translate_time")
				.as_date("yyyy-MM-dd HH:mm:ss", None, None)
				.unwrap_or(-1.0);
			Ok(Chapter {
				id: chapter.to_string(),
				title: stripobj.get("title").as_string()?.read(),
				volume: -1.0,
				chapter: chapter as f32,
				date_updated,
				scanlator: String::new(),
				url: format!("https://xkcd.tw/{chapter}"),
				lang: String::from("zh"),
			})
		})
		.filter_map(|val: Result<Chapter>| val.ok())
		.collect::<Vec<_>>())
}

pub fn get_page_list(id: String) -> Result<Vec<Page>> {
	super::get_page_list(
		format!("https://xkcd.tw/{id}"),
		String::from("#content > img:not([id])"),
		// Google translated, sorry
		true,
		format!("要體驗本漫畫的互動版\n請在瀏覽器中打開: https://xkcd.tw/{id}/"),
		super::ImageVariant::Cjk,
	)
}
