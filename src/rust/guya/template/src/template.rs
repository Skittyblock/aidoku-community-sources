use aidoku::{
	error::Result, prelude::*, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Filter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer,
};
extern crate alloc;

pub struct GuyaSiteData {
	pub base_url: String,
}

pub fn get_manga_list(
	data: GuyaSiteData,
	filters: Vec<Filter>,
	page: i32,
) -> Result<MangaPageResult> {
	let url = format!("{}/api/get_all_series/", &data.base_url);
	let request = Request::new(url, HttpMethod::Get).header("User-Agent", "Aidoku");
	let json = request.json()?.as_object()?;

	let mut manga_arr: Vec<Manga> = Vec::new();
	for manga in json.keys() {
		let title = manga.as_string()?.read();
		let obj = match json.get(&title).as_object() {
			Ok(obj) => obj,
			Err(_) => continue,
		};
		let id = obj.get("slug").as_string()?.read();
		let cover = format!("{}{}", &data.base_url, obj.get("cover").as_string()?.read());
		manga_arr.push(Manga {
			id,
			title,
			cover,
			status: MangaStatus::Unknown,
			nsfw: MangaContentRating::Safe,
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

// pub fn get_manga_details(todo!()) -> Result<Manga> {
// 	todo!()
// }

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
