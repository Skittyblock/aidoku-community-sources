#![no_std]
use aidoku::{
	error::Result, helpers::uri::encode_uri_component, prelude::*, std::defaults::defaults_get,
	std::net::HttpMethod, std::net::Request, std::ObjectRef, std::String, std::Vec, Chapter,
	DeepLink, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};
extern crate alloc;
use alloc::{string::ToString, vec};

mod helper;

const BASE_URL: &str = "https://nhentai.net";
const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/604";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut sauce_code = Option::<String>::None;

	let mut query = if let Ok(languages) = defaults_get("languages")?.as_array() {
		if languages.is_empty() {
			String::from("language:english")
		} else {
			languages
				.into_iter()
				.filter_map(|lang| lang.as_string().ok())
				.map(|lang| match lang.read().as_str() {
					"en" => "language:english",
					"ja" => "language:japanese",
					"zh" => "language:chinese",
					"All" => "\"\"",
					_ => "",
				})
				.collect::<Vec<&str>>()
				.join(" ")
		}
	} else {
		String::from("language:english")
	};

	let mut sort = "date";

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				let title = filter.value.as_string()?.read();
				if helper::is_number(&title) {
					sauce_code = Some(title.clone());
				} else {
					query.push(' ');
					query.push_str(&title);
				}
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
				sort = option;
			}
			_ => continue,
		}
	}

	// if the user searches a code, just return the manga for that id
	if let Some(sauce_code) = sauce_code {
		let manga = get_manga_details(sauce_code)?;

		return Ok(MangaPageResult {
			manga: vec![manga],
			has_more: false,
		});
	}

	let url = format!(
		"{BASE_URL}/search/?q={}&page={page}&sort={sort}",
		encode_uri_component(query),
	);

	let html = Request::new(&url, HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.html()?;

	let mut manga: Vec<Manga> = Vec::new();

	for node in html
		.select("#content .container:not(.index-popular) .gallery")
		.array()
	{
		let node = node.as_node().expect("node array");

		let rel_link = node.select("a").first().attr("href").read(); // /g/id/

		let id = rel_link[3..rel_link.len() - 1].to_string();

		let cover = node.select("img").first().attr("data-src").read();

		let title = node.select(".caption").first().text().read();

		manga.push(Manga {
			id,
			cover: format!("https:{}", cover),
			title,
			nsfw: MangaContentRating::Nsfw,
			..Default::default()
		});
	}

	let has_more = !manga.is_empty();

	Ok(MangaPageResult { manga, has_more })
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
			_ => 0i32.into(),
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
	let url = format!("{BASE_URL}/g/{id}/");
	let html = Request::new(&url, HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.html()?;

	let cover = html.select("#cover img").first().attr("data-src").read();

	let title = html.select("#info h1.title").first().text().read();

	let author = html
		.select("#tags div:contains(Artists:)")
		.array()
		.map(|node| {
			node.as_node()
				.expect("node array")
				.select(".name")
				.first()
				.text()
				.read()
		})
		.collect::<Vec<_>>()
		.join(", ");
	let artist = author.clone();

	let description = format!("#{id}");

	let categories = html
		.select("#tags div:contains(Tags:) a")
		.array()
		.map(|node| {
			node.as_node()
				.expect("node array")
				.select(".name")
				.first()
				.text()
				.read()
		})
		.collect();

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
	let json = Request::new(helper::get_details_url(id.clone()), HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.json()?
		.as_object()?;

	let url = format!("{BASE_URL}/g/{id}/");

	let date_updated = json.get("upload_date").as_float().unwrap_or(0.0);

	let language = &helper::get_tag_names_by_type(json.get("tags").as_array()?, "artist")?[0];

	let lang = match language.as_str() {
		"english" => "en",
		"japanese" => "jp",
		"chinese" => "zh",
		_ => "",
	}
	.to_string();

	Ok(vec![Chapter {
		id,
		chapter: 1.0,
		date_updated,
		url,
		lang,
		..Default::default()
	}])
}

#[get_page_list]
fn get_page_list(_: String, id: String) -> Result<Vec<Page>> {
	let html = Request::new(format!("{BASE_URL}/g/{id}/"), HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.string()?;
	let media_server = helper::find_media_server(&html).unwrap_or("");

	let json = Request::new(helper::get_details_url(id).as_str(), HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.json()?
		.as_object()?;

	let images = json.get("images").as_object()?;
	let pages_arr = images.get("pages").as_array()?;

	let mut pages = Vec::new();

	for (i, page) in pages_arr.enumerate() {
		let page_obj = page.as_object()?;

		let media_id = json.get("media_id").as_string()?.read();
		let file_type = helper::get_file_type(page_obj.get("t").as_string()?.read());

		let url = format!(
			"https://i{media_server}.nhentai.net/galleries/{media_id}/{}.{file_type}",
			i + 1
		);

		pages.push(Page {
			index: i as i32,
			url,
			..Default::default()
		});
	}

	Ok(pages)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let url = &url[BASE_URL.len()..]; // remove "https://nhentai.net"

	if let Some(id) = url.strip_prefix("/g/") {
		let end = match id.find('/') {
			Some(end) => end,
			None => id.len(),
		};
		let manga_id = id[..end].to_string();
		let manga = get_manga_details(manga_id)?;

		return Ok(DeepLink {
			manga: Some(manga),
			chapter: None,
		});
	}

	Err(aidoku::error::AidokuError {
		reason: aidoku::error::AidokuErrorKind::Unimplemented,
	})
}
