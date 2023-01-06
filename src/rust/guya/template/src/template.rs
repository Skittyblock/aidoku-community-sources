use aidoku::{
	error::Result, prelude::*, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer,
};
use alloc::string::ToString;
extern crate alloc;

pub struct GuyaSiteData {
	pub base_url: String,
	pub nsfw: MangaContentRating,
}

impl Default for GuyaSiteData {
	fn default() -> GuyaSiteData {
		GuyaSiteData {
			base_url: String::new(),
			nsfw: MangaContentRating::Safe,
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

// pub fn get_chapter_list(todo!()) -> Result<Vec<Chapter>> {
// 	todo!()
// }

// pub fn get_page_list(todo!()) -> Result<Vec<Page>> {
// 	todo!()
// }

// pub fn modify_image_request(todo!(), request: Request) {
// 	todo!()
// }

// pub fn handle_url(todo!()) -> Result<DeepLink> {
// 	todo!()
// }
