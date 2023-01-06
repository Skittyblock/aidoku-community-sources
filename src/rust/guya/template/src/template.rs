use aidoku::{
	error::Result, prelude::*, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer,
};
use alloc::string::ToString;
extern crate alloc;

pub struct GuyaSiteData {
	pub base_url: String,
	pub nsfw: MangaContentRating,
	pub language: String,
}

impl Default for GuyaSiteData {
	fn default() -> GuyaSiteData {
		GuyaSiteData {
			base_url: String::new(),
			nsfw: MangaContentRating::Safe,
			language: String::from("en"),
		}
	}
}

pub fn get_manga_list(data: GuyaSiteData, filters: Vec<Filter>, _: i32) -> Result<MangaPageResult> {
	let url = format!("{}/api/get_all_series/", &data.base_url);
	let request = Request::new(url, HttpMethod::Get).header("User-Agent", "Aidoku");
	let mut json = request.json()?.as_object()?;

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				let query = filter.value.as_string()?.read();
				for manga in json.keys() {
					let title = manga.as_string()?.read();
					if !title.to_lowercase().contains(&query.to_lowercase()) {
						json.remove(&title);
					}
				}
			}
			_ => continue,
		}
	}

	let mut manga_arr: Vec<Manga> = Vec::new();
	for manga in json.keys() {
		let title = manga.as_string()?.read();
		let obj = match json.get(&title).as_object() {
			Ok(obj) => obj,
			Err(_) => continue,
		};
		let slug = obj.get("slug").as_string()?.read();
		let cover = format!("{}{}", &data.base_url, obj.get("cover").as_string()?.read());
		let description = obj.get("description").as_string()?.read();
		let author = obj.get("author").as_string()?.read();
		let artist = obj.get("artist").as_string()?.read();
		let user_url = format!("{}/read/manga/{}/", &data.base_url, slug);
		manga_arr.push(Manga {
			id: slug,
			title,
			cover,
			description,
			author,
			artist,
			url: user_url,
			status: MangaStatus::Unknown,
			nsfw: data.nsfw,
			viewer: MangaViewer::Rtl,
			..Default::default()
		})
	}

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: false,
	})
}

// pub fn get_manga_listing(todo!()) -> Result<MangaPageResult> {
// 	todo!()
// }

pub fn get_manga_details(_: GuyaSiteData, _: String) -> Result<Manga> {
	Ok(Manga {
		..Default::default()
	})
}

pub fn get_chapter_list(data: GuyaSiteData, slug: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/api/series/{}/", &data.base_url, slug);
	let request = Request::new(url, HttpMethod::Get).header("User-Agent", "Aidoku");
	let json = request.json()?.as_object()?;
	let mut chapter_arr: Vec<Chapter> = Vec::new();
	let slug = json.get("slug").as_string()?.read();
	let chapter_obj = json.get("chapters").as_object()?;
	let mut chapters: Vec<f32> = chapter_obj
		.keys()
		.map(|k| k.as_string().unwrap().read().parse::<f32>().unwrap())
		.collect();
	chapters.sort_by(|a, b| b.partial_cmp(a).unwrap());
	for chapter in chapters {
		let chapter = chapter.to_string();
		let obj = match chapter_obj.get(&chapter).as_object() {
			Ok(obj) => obj,
			Err(_) => continue,
		};
		let title = obj.get("title").as_string()?.read();
		let volume = obj
			.get("volume")
			.as_string()?
			.read()
			.parse()
			.unwrap_or(-1.0);
		let chapter_int = chapter.parse().unwrap_or(1.0);
		let user_url = format!("{}/read/manga/{}/{}/", &data.base_url, &slug, chapter);
		for groups in obj.get("groups").as_object()?.keys() {
			let group_id = groups.as_string()?.read();
			let mut group_name = String::new();
			let group_list = json.get("groups").as_object()?;
			for gl_index in group_list.keys() {
				let gl_id = gl_index.as_string()?.read();
				if gl_id == group_id {
					group_name = group_list.get(&gl_id).as_string()?.read();
				}
			}
			let mut date_updated = 0.0;
			let date_list = obj.get("release_date").as_object()?;
			for dl_index in date_list.keys() {
				let dl_id = dl_index.as_string()?.read();
				if dl_id == group_id {
					date_updated = date_list.get(&dl_id).as_float()?;
				}
			}
			let slug = &slug;
			let title = &title;
			let url = &user_url;
			let language = &data.language;
			chapter_arr.push(Chapter {
				id: slug.to_string(),
				title: title.to_string(),
				volume,
				chapter: chapter_int,
				scanlator: group_name,
				date_updated,
				url: url.to_string(),
				lang: language.to_string(),
			})
		}
	}

	Ok(chapter_arr)
}

// pub fn get_page_list(todo!()) -> Result<Vec<Page>> {
// 	todo!()
// }

// pub fn modify_image_request(todo!(), request: Request) {
// 	todo!()
// }

// pub fn handle_url(todo!()) -> Result<DeepLink> {
// 	todo!()
// }
