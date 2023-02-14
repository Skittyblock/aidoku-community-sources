use aidoku::{
	error::Result,
	prelude::*,
	std::net::HttpMethod,
	std::net::Request,
	std::String,
	std::{html::Node, ObjectRef, Vec},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
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
	let mut json = request
		.json()
		.expect("Manga list json not found")
		.as_object()?;

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
	let mut titles: Vec<String> = json.keys().map(|k| k.as_string().unwrap().read()).collect();
	titles.sort();
	for title in titles {
		let obj = match json.get(&title).as_object() {
			Ok(obj) => obj,
			Err(_) => continue,
		};
		let slug = match obj.get("slug").as_string() {
			Ok(slug) => slug.read(),
			Err(_) => continue,
		};
		let cover = format!("{}{}", &data.base_url, obj.get("cover").as_string()?.read());
		manga_arr.push(Manga {
			id: slug,
			title,
			cover,
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

pub fn get_manga_details(
	data: GuyaSiteData,
	slug: String,
	nsfw: MangaContentRating,
) -> Result<Manga> {
	let url = format!("{}/api/series/{}/", &data.base_url, slug);
	let request = Request::new(url, HttpMethod::Get).header("User-Agent", "Aidoku");
	let json = request
		.json()
		.expect("Manga detail json not found")
		.as_object()?;
	let title = json
		.get("title")
		.as_string()
		.expect("Manga detail title not found")
		.read();
	let cover = format!(
		"{}{}",
		&data.base_url,
		json.get("cover").as_string()?.read()
	);
	let description_raw = json.get("description").as_string()?.read();
	let description_node = Node::new_fragment(description_raw.as_bytes())?;
	let description = match description_node.select("body").array().get(0).as_node() {
		Ok(node) => node.own_text().read(),
		Err(_) => String::from(""),
	};
	let user_url = format!("{}/read/manga/{}/", &data.base_url, slug);
	let author = match json.get("author").as_string() {
		Ok(author) => author.read(),
		Err(_) => String::from("Unknown Author"),
	};
	let artist = match json.get("artist").as_string() {
		Ok(artist) => artist.read(),
		Err(_) => String::from("Unknown Artist"),
	};
	Ok(Manga {
		id: slug,
		title,
		cover,
		description,
		author,
		artist,
		url: user_url,
		status: MangaStatus::Unknown,
		nsfw,
		viewer: MangaViewer::Rtl,
		..Default::default()
	})
}

pub fn get_chapter_list(data: GuyaSiteData, slug: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/api/series/{}/", &data.base_url, slug);
	let request = Request::new(url, HttpMethod::Get).header("User-Agent", "Aidoku");
	let json = request
		.json()
		.expect("Manga chapter list json not found")
		.as_object()?;
	let mut chapter_arr: Vec<Chapter> = Vec::new();
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
		let folder = obj.get("folder").as_string()?.read();
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
			let id = format!("{}|{}", &folder, &group_id);
			chapter_arr.push(Chapter {
				id,
				title: title.clone(),
				volume,
				chapter: chapter_int,
				scanlator: group_name,
				date_updated,
				url: user_url.clone(),
				lang: data.language.clone(),
			});
		}
	}

	Ok(chapter_arr)
}

pub fn get_page_list(data: GuyaSiteData, chapter: ObjectRef) -> Result<Vec<Page>> {
	let slug = chapter
		.get("mangaId")
		.as_string()
		.expect("Manga chapter object slug not found")
		.read();
	let url = format!("{}/api/series/{}/", &data.base_url, &slug);
	let request = Request::new(url, HttpMethod::Get).header("User-Agent", "Aidoku");
	let json = request
		.json()
		.expect("Manga page list json not found")
		.as_object()?;
	let chapter_num = chapter.get("chapterNum").as_float()?;
	let chapter_num = format!("{:.1}", chapter_num)
		.trim_end_matches(".0")
		.to_string();

	let ids = chapter.get("id").as_string()?.read();
	let group_id = ids.split('|').collect::<Vec<&str>>()[1].to_string();
	let chapters_obj = json.get("chapters").as_object()?;
	let chapter_obj = chapters_obj.get(chapter_num.as_str()).as_object()?;
	let folder = chapter_obj.get("folder").as_string()?.read();
	let groups_obj = chapter_obj.get("groups").as_object()?;
	let chapter_array = groups_obj.get(group_id.as_str()).as_array()?;
	let mut pages: Vec<Page> = Vec::new();
	for (idx, page) in chapter_array.enumerate() {
		let page_string = page.as_string()?.read();
		let page_url = format!(
			"{}/media/manga/{}/chapters/{}/{}/{}",
			&data.base_url, &slug, folder, group_id, page_string
		);
		pages.push(Page {
			index: idx as i32,
			url: page_url,
			..Default::default()
		});
	}

	Ok(pages)
}

pub fn handle_url(data: GuyaSiteData, url: String, nsfw: MangaContentRating) -> Result<DeepLink> {
	let parts = url.split('/').collect::<Vec<&str>>();
	let slug = parts[5].to_string();
	let manga = get_manga_details(data, slug, nsfw).ok();
	Ok(DeepLink {
		manga,
		chapter: None,
	})
}
