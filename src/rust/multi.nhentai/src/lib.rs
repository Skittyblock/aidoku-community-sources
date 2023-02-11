#![no_std]
use aidoku::{
	error::Result, prelude::*, std::defaults::defaults_get, std::net::HttpMethod,
	std::net::Request, std::ObjectRef, std::String, std::Vec, Chapter, DeepLink, Filter,
	FilterType, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer,
	Page,
};
extern crate alloc;
use alloc::{string::ToString, vec};

mod helper;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query: String;
	let mut is_sauce_code = false;
	let mut sauce_code = String::from("0");

	if let Ok(languages) = defaults_get("languages")?.as_array() {
		query = String::new();
		if languages.is_empty() {
			query.push_str("language:english")
		} else {
			for lang in languages {
				match lang.as_string()?.read().as_str() {
					"en" => query.push_str("language:english"),
					"jp" => query.push_str("language:japanese"),
					"zh" => query.push_str("language:chinese"),
					_ => {}
				}
			}
		}
	} else {
		query = String::from("language:english")
	}

	let mut sort = String::from("date");

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				let title = filter.value.as_string()?.read();
				if helper::is_number(title.as_str()) {
					is_sauce_code = true;
					sauce_code = title.clone();
				}
				query.push(' ');
				query.push_str(&title);
			}
			FilterType::Genre => {
				match filter.value.as_int().unwrap_or(-1) {
					0 => query.push_str(" -tag:\""),
					1 => query.push_str(" tag:\""),
					_ => continue,
				}
				query.push_str(&filter.name);
				query.push('\"');
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int().unwrap_or(0);
				let option = match index {
					0 => "date",
					1 => "popular-today",
					2 => "popular-week",
					3 => "popular",
					_ => continue,
				};
				sort = String::from(option)
			}
			_ => continue,
		}
	}

	let mut manga_arr: Vec<Manga> = Vec::new();
	let mut total: i32 = 1;

	if is_sauce_code {
		let url = helper::get_details_url(sauce_code);
		let request = Request::new(url, HttpMethod::Get).header("User-Agent", "Aidoku");
		let json = request.json()?.as_object()?;

		let id = helper::get_id(json.get("id"))?;

		let media_id = json.get("media_id").as_string()?.read();
		let cover_type = json
			.get("images")
			.as_object()?
			.get("cover")
			.as_object()?
			.get("t")
			.as_string()?
			.read();
		let cover = helper::get_cover_url(media_id, helper::get_file_type(cover_type));

		let title = json
			.get("title")
			.as_object()?
			.get("pretty")
			.as_string()?
			.read();

		manga_arr.push(Manga {
			id,
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: String::new(),
			categories: Vec::new(),
			status: MangaStatus::Completed,
			nsfw: MangaContentRating::Nsfw,
			viewer: MangaViewer::Rtl,
		});
	} else {
		let mut url = String::from("https://nhentai.net/api/galleries/search?query=");
		url.push_str(&helper::urlencode(query));
		url.push_str("&page=");
		url.push_str(&helper::urlencode(page.to_string()));
		url.push_str("&sort=");
		url.push_str(&helper::urlencode(sort));

		let request = Request::new(&url, HttpMethod::Get).header("User-Agent", "Aidoku");
		let json = request.json()?.as_object()?;

		let data = json.get("result").as_array()?;

		for manga in data {
			let manga_obj = manga.as_object()?;

			let id = helper::get_id(manga_obj.get("id"))?;

			let media_id = manga_obj.get("media_id").as_string()?.read();
			let cover_type = manga_obj
				.get("images")
				.as_object()?
				.get("cover")
				.as_object()?
				.get("t")
				.as_string()?
				.read();
			let cover = helper::get_cover_url(media_id, helper::get_file_type(cover_type));

			let title = manga_obj
				.get("title")
				.as_object()?
				.get("pretty")
				.as_string()?
				.read();

			manga_arr.push(Manga {
				id,
				cover,
				title,
				author: String::new(),
				artist: String::new(),
				description: String::new(),
				url: String::new(),
				categories: Vec::new(),
				status: MangaStatus::Completed,
				nsfw: MangaContentRating::Nsfw,
				viewer: MangaViewer::Rtl,
			});
		}
		total = json.get("num_pages").as_int().unwrap_or(0) as i32;
	}

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: page < total,
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let mut filters: Vec<Filter> = Vec::new();
	let mut selection = ObjectRef::new();

	selection.set("ascending", false.into());
	selection.set(
		"index",
		match listing.name.as_str() {
			"Latest" => 0i32.into(),
			"Popular - Today" => 1i32.into(),
			"Popular - This Week" => 2i32.into(),
			"Popular - All Time" => 3i32.into(),
			&_ => 0i32.into(),
		},
	);

	filters.push(Filter {
		kind: FilterType::Sort,
		name: String::from("Sort"),
		value: selection.0,
		object: ObjectRef::new(),
	});

	get_manga_list(filters, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let request = Request::new(helper::get_details_url(id).as_str(), HttpMethod::Get)
		.header("User-Agent", "Aidoku");
	let json = request.json()?.as_object()?;

	let id = helper::get_id(json.get("id"))?;

	let media_id = json.get("media_id").as_string()?.read();
	let cover_type = json
		.get("images")
		.as_object()?
		.get("cover")
		.as_object()?
		.get("t")
		.as_string()?
		.read();
	let cover = helper::get_cover_url(media_id, helper::get_file_type(cover_type));

	let title = json
		.get("title")
		.as_object()?
		.get("english")
		.as_string()?
		.read();

	let tags = json.get("tags").as_array()?;
	let author = String::from(helper::get_tag_names_by_type(tags.clone(), "artist")?[0].as_str());
	let artist = author.clone();

	let mut url = String::from("https://nhentai.net/g/");
	url.push_str(&id);

	let mut description = String::from("#");
	description.push_str(&id);

	let categories = helper::get_tag_names_by_type(tags, "tag")?;

	Ok(Manga {
		id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories,
		status: MangaStatus::Completed,
		nsfw: MangaContentRating::Nsfw,
		viewer: MangaViewer::Rtl,
	})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let request = Request::new(
		helper::get_details_url(id.clone()).as_str(),
		HttpMethod::Get,
	)
	.header("User-Agent", "Aidoku");
	let json = request.json()?.as_object()?;

	let mut url = String::from("https://nhentai.net/g/");
	url.push_str(&id);

	let date_updated = json.get("upload_date").as_float().unwrap_or(0.0);

	let language = &helper::get_tag_names_by_type(json.get("tags").as_array()?, "artist")?[0];

	let lang = match language.as_str() {
		"english" => String::from("en"),
		"japanese" => String::from("jp"),
		"chinese" => String::from("zh"),
		_ => String::new(),
	};

	Ok(vec![Chapter {
		id,
		title: String::from("Chapter 1"),
		volume: -1.0,
		chapter: 1.0,
		date_updated,
		scanlator: String::new(),
		url,
		lang,
	}])
}

#[get_page_list]
fn get_page_list(_: String, id: String) -> Result<Vec<Page>> {
	let request = Request::new(helper::get_details_url(id).as_str(), HttpMethod::Get)
		.header("User-Agent", "Aidoku");
	let json = request.json()?.as_object()?;

	let images = json.get("images").as_object()?;
	let pages_arr = images.get("pages").as_array()?;

	let mut pages = Vec::new();

	for (i, page) in pages_arr.enumerate() {
		let page_obj = page.as_object()?;

		let media_id = json.get("media_id").as_string()?.read();
		let file_type = helper::get_file_type(page_obj.get("t").as_string()?.read());

		let mut url = String::from("https://i.nhentai.net/galleries/");
		url.push_str(&media_id);
		url.push('/');
		url.push_str(&(i + 1).to_string());
		url.push('.');
		url.push_str(&file_type);

		pages.push(Page {
			index: i.try_into().unwrap_or(-1),
			url,
			base64: String::new(),
			text: String::new(),
		});
	}

	Ok(pages)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let url = &url[20..]; // remove "https://nhentai.net/"

	if let Some(id) = url.strip_prefix("g/") {
		let end = match id.find('/') {
			Some(end) => end,
			None => id.len(),
		};
		let manga_id = &id[..end];
		let manga = get_manga_details(String::from(manga_id))?;

		return Ok(DeepLink {
			manga: Some(manga),
			chapter: None,
		});
	}

	Err(aidoku::error::AidokuError {
		reason: aidoku::error::AidokuErrorKind::Unimplemented,
	})
}
