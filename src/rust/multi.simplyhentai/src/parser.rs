use aidoku::{
	error::Result, prelude::format, std::ObjectRef, Chapter, Manga, MangaContentRating,
	MangaPageResult, MangaStatus, Page,
};
use alloc::{
	string::{String, ToString},
	vec::Vec,
};

use crate::helper::{get_image_quality, BASE_URL};

pub fn parse_search(page: i32, res: ObjectRef) -> Result<MangaPageResult> {
	let mut manga_arr = Vec::new();
	let has_more = res.get("pagination").as_object()?.get("pages").as_int()? > page.into();

	let list = res.get("data").as_array()?;
	let image_quality = get_image_quality()?;
	for itemref in list {
		let object = itemref.as_object()?;
		let data = object.get("object").as_object()?;

		let id = data.get("id").as_int()?.to_string();
		let title = data.get("title").as_string()?.read();
		let cover = data
			.get("preview")
			.as_object()?
			.get("sizes")
			.as_object()?
			.get(&image_quality)
			.as_string()?
			.read();

		manga_arr.push(Manga {
			id,
			title,
			cover,
			..Manga::default()
		});
	}

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more,
	})
}

fn handle_resource_title(data: &ObjectRef, key: &str) -> Result<Vec<String>> {
	let resources = data.get(key).as_array()?;
	let mut titles = Vec::new();
	for resource in resources {
		let resource = resource.as_object()?;
		let title = resource.get("title").as_string()?;
		titles.push(title.read());
	}
	Ok(titles)
}

pub fn parse_manga(id: String, res: ObjectRef) -> Result<Manga> {
	let data = res.get("data").as_object()?;
	let title = data.get("title").as_string()?.read();
	let image_quality = get_image_quality()?;

	let cover = data
		.get("preview")
		.as_object()?
		.get("sizes")
		.as_object()?
		.get(&image_quality)
		.as_string()?
		.read();
	let author = handle_resource_title(&data, "artists")?.join(", ");
	let categories = handle_resource_title(&data, "tags")?;

	let series_slug = data
		.get("series")
		.as_object()?
		.get("slug")
		.as_string()?
		.read();
	let slug = data.get("slug").as_string()?.read();
	let url = format!("{BASE_URL}/{series_slug}/{slug}");

	Ok(Manga {
		id,
		cover,
		title,
		author,
		url,
		categories,
		status: MangaStatus::Completed,
		nsfw: MangaContentRating::Nsfw,
		..Manga::default()
	})
}

pub fn parse_chapter_list(manga_id: String, res: ObjectRef) -> Result<Vec<Chapter>> {
	let data = res.get("data").as_object()?;
	let date_updated =
		data.get("created_at")
			.as_date("yyyy-MM-dd'T'HH:mm:ss+ss:ss", None, Some("UTC"))?;
	Ok(Vec::from([Chapter {
		id: manga_id,
		chapter: 1.0,
		date_updated,
		..Chapter::default()
	}]))
}

pub fn parse_page_list(res: ObjectRef) -> Result<Vec<Page>> {
	let data = res.get("data").as_object()?;
	let images = data.get("pages").as_array()?;
	let mut pages = Vec::new();
	let image_quality = get_image_quality()?;

	for imageref in images {
		let image = imageref.as_object()?;
		let i = image.get("page_num").as_int()? as i32;
		let index = i - 1;
		let sizes = image.get("sizes").as_object()?;
		let url = sizes.get(&image_quality).as_string()?.read();
		pages.push(Page {
			index,
			url,
			..Page::default()
		});
	}
	Ok(pages)
}
