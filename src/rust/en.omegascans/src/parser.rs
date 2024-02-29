use aidoku::{
	error::Result, prelude::format, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Chapter, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page, std::current_date, helpers::uri::encode_uri_component
};

pub fn parse_manga_list(base_url: String, filters: Vec<Filter>, _page: i32) -> Result<MangaPageResult> {
	
	let mut search_query = String::new();
	let mut status = String::new();
	let mut genres = String::new();
	
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					search_query.push_str(&encode_uri_component(filter_value.read()));
				}
			}
			FilterType::Genre => {
				if let Ok(filter_id) = filter.object.get("id").as_string() {
					match filter.value.as_int().unwrap_or(-1) {
						1 => {
							genres.push_str(filter_id.read().as_str());
							genres.push(',');
						}
						_ => continue,
					}
				}
			}
			FilterType::Select => match filter.name.as_str() {
				"Status" => match filter.value.as_int().unwrap_or(-1) {
					0 => status.push_str("All"),
					1 => status.push_str("Ongoing"),
					2 => status.push_str("Hiatus"),
					3 => status.push_str("Dropped"),
					4 => status.push_str("Completed"),
					5 => status.push_str("Cancelled"),
					_ => continue,
				},
				_ => continue,
			}
			_ => continue,
		}
	}

	if !genres.is_empty() {
		genres.pop();
	}

	let url = format!("https://api.omegascans.org/query?query_string={}&series_status={}&order=desc&orderBy=total_views&series_type=Comic&page=1&perPage=1000&tags_ids=[{}]", search_query, status, genres);

	let json = Request::new(url, HttpMethod::Get)
		.json()
		.expect("Failed to load JSON")
		.as_object()
		.expect("Failed to get JSON as object");

	let data = json
		.get("data")
		.as_array()
		.expect("Failed to get data as object");

	let mut mangas: Vec<Manga> = Vec::new();

	for manga in data {
		let manga = manga.as_object().expect("Failed to get manga as object");

		let title = manga
			.get("title")
			.as_string()
			.expect("Failed to get manga name as str")
			.read();

		let cover = manga
			.get("thumbnail")
			.as_string()
			.expect("Failed to get manga cover as str")
			.read();

		let slug = manga
			.get("series_slug")
			.as_string()
			.expect("Failed to get manga slug as str")
			.read();

		let status = manga
			.get("status")
			.as_string()
			.expect("Failed to get manga status as array")
			.read();

		let mut manga_status = MangaStatus::Unknown;

		match status.as_str() {
			"New" => {}
			"Ongoing" => manga_status = MangaStatus::Ongoing,
			"Completed" => manga_status = MangaStatus::Completed,
			"Dropped" => manga_status = MangaStatus::Cancelled,
			"Hiatus" => manga_status = MangaStatus::Hiatus,
			_ => manga_status = MangaStatus::Unknown,
		};


		let url = format!("{}/series/{}", base_url, slug);

		mangas.push(Manga {
			id: slug,
			cover: cover,
			title: title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: url,
			categories: Vec::new(),
			status: manga_status,
			nsfw: MangaContentRating::Nsfw,
			viewer: MangaViewer::Scroll,
		});
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more: false,
	})
}

pub fn parse_manga_listing(base_url: String, url: String, _listing: Listing, _page: i32) -> Result<MangaPageResult> {
	
	let json = Request::new(url, HttpMethod::Get)
		.json()
		.expect("Failed to load JSON")
		.as_object()
		.expect("Failed to get JSON as object");

	let data = json
		.get("data")
		.as_array()
		.expect("Failed to get data as object");

	let mut mangas: Vec<Manga> = Vec::new();

	for manga in data {
		let manga = manga.as_object().expect("Failed to get manga as object");

		let title = manga
			.get("title")
			.as_string()
			.expect("Failed to get manga name as str")
			.read();

		let cover = manga
			.get("thumbnail")
			.as_string()
			.expect("Failed to get manga cover as str")
			.read();

		let slug = manga
			.get("series_slug")
			.as_string()
			.expect("Failed to get manga slug as str")
			.read();

		let status = manga
			.get("status")
			.as_string()
			.expect("Failed to get manga status as array")
			.read();

		let mut manga_status = MangaStatus::Unknown;

		match status.as_str() {
			"New" => {}
			"Ongoing" => manga_status = MangaStatus::Ongoing,
			"Completed" => manga_status = MangaStatus::Completed,
			"Cancelled" => manga_status = MangaStatus::Cancelled,
			"Dropped" => manga_status = MangaStatus::Cancelled,
			"Hiatus" => manga_status = MangaStatus::Hiatus,
			_ => manga_status = MangaStatus::Unknown,
		};


		let url = format!("{}/series/{}", base_url, slug);

		mangas.push(Manga {
			id: slug,
			cover: cover,
			title: title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: url,
			categories: Vec::new(),
			status: manga_status,
			nsfw: MangaContentRating::Nsfw,
			viewer: MangaViewer::Scroll,
		});
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more: false,
	})
}

pub fn parse_manga_details(base_url: String, manga_id: String) -> Result<Manga> {
	let url = format!("https://api.omegascans.org/series/{}", manga_id);

	let data = Request::new(url, HttpMethod::Get)
		.json()
		.expect("Failed to load JSON")
		.as_object()
		.expect("Failed to get JSON as object");

	let cover = data
		.get("thumbnail")
		.as_string()
		.expect("Failed to get manga cover as str")
		.read();

	let title = data
		.get("title")
		.as_string()
		.expect("Failed to get manga name as str")
		.read();

	let description = data
		.get("description")
		.as_string()
		.expect("Failed to get manga summary as str")
		.read();

	let author = data
		.get("author")
		.as_string()
		.expect("Failed to get author name as str")
		.read();

	let artist = data
		.get("studio")
		.as_string()
		.expect("Failed to get artist name as str")
		.read();

	let slug = manga_id;

	let url = format!("{}/series/{}", base_url, slug);

	let status = data
		.get("status")
		.as_string()
		.expect("Failed to get manga string as array")
		.read();

	let mut manga_status = MangaStatus::Unknown;

	match status.as_str() {
		"New" => {}
		"Ongoing" => manga_status = MangaStatus::Ongoing,
		"Completed" => manga_status = MangaStatus::Completed,
		"Cancelled" => manga_status = MangaStatus::Cancelled,
		"Dropped" => manga_status = MangaStatus::Cancelled,
		"Hiatus" => manga_status = MangaStatus::Hiatus,
		_ => manga_status = MangaStatus::Unknown,
	};

	Ok(Manga {
		id: slug,
		cover: cover,
		title: title,
		author: author,
		artist: artist,
		description: description,
		url: url,
		categories: Vec::new(),
		status: manga_status,
		nsfw: MangaContentRating::Nsfw,
		viewer: MangaViewer::Scroll,
	})
}

pub fn parse_chapter_list(base_url: String, manga_id: String) -> Result<Vec<Chapter>> {

	let url = format!("https://api.omegascans.org/series/{}", manga_id);

	let data = Request::new(url, HttpMethod::Get)
		.json()
		.expect("Failed to load JSON")
		.as_object()
		.expect("Failed to get JSON as object");

	let mut all_chapters: Vec<Chapter> = Vec::new();

	let seasons = data.get("seasons").as_array().expect("Failed to get seasons as array");

	for season in seasons {
		let season = season.as_object().expect("Failed to get season as object");
		let volume = season.get("index").as_int().expect("Failed to get name as str");
		
		let chapters = season.get("chapters").as_array().expect("Failed to get chapters as array");

		for chapter in chapters {
			let chapter = chapter.as_object().expect("Failed to get chapter as object");

			let price = chapter.get("price").as_int().expect("Failed to get chapter price");

			if price != 0 {
				continue;
			}

			let name = chapter.get("chapter_name").as_string().expect("Failed to get chapter name as str").read();
			let index = chapter.get("index").as_string().expect("Failed to get chapter index as int").read();
			let chapter_slug = chapter.get("chapter_slug").as_string().expect("Failed to get chapter slug as str").read();
			let url = format!("{}/series/{}/{}", base_url, manga_id, chapter_slug);
			let date_updated = chapter.get("created_at").as_string().expect("").read();
			//aidoku::prelude::println!("{}", date_updated);
			let date_updated = get_date(date_updated);

			all_chapters.push(Chapter {
				id: chapter_slug,
				title: name,
				volume: volume as f32,
				chapter: index.parse::<f32>().unwrap_or_default(),
				date_updated: date_updated,
				scanlator: String::new(),
				url: url,
				lang: String::from("en"),
			});
			
			

		}
	}
		

	Ok(all_chapters)
}

pub fn parse_page_list(base_url: String, manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{}/series/{}/{}", base_url, manga_id, chapter_id);

	let obj = Request::new(url.as_str(), HttpMethod::Get).html()?;

	let mut page_list: Vec<Page> = Vec::new();
	
	for (i, page) in obj.select("img").array().enumerate() {
		let obj = page.as_node().expect("node array");
		let url = obj.attr("data-src").read();
		let url2 = obj.attr("src").read();

		if url != "" {
			page_list.push(Page {
				index: i as i32,
				url: url,
				text: String::new(),
				base64: String::new(),
			});
		}

		else if url2 != "" {
			page_list.push(Page {
				index: i as i32,
				url: url2,
				text: String::new(),
				base64: String::new(),
			});
		}

	}

	// Remove icon.png and banners from top and bottom
	page_list.remove(0);
	page_list.remove(0);
    page_list.pop();
	page_list.pop();

	Ok(page_list)
}

pub fn modify_image_request(base_url: String, request: Request) {
	request.header("Referer", &base_url);
}

pub fn get_date(time_ago: String) -> f64 {
	let cleaned_time_ago = String::from(time_ago.replace("Released", "").replace("ago", "").trim());

	let number = cleaned_time_ago
		.split_whitespace()
		.next()
		.unwrap_or("")
		.parse::<f64>()
		.unwrap_or(0.0);

	match cleaned_time_ago
		.to_uppercase()
		.split_whitespace()
		.last()
		.unwrap_or("")
	{
		"YEAR" | "YEARS" => current_date() - (number * 60.0 * 60.0 * 24.0 * 365.0),
		"MONTH" | "MONTHS" => current_date() - (number * 60.0 * 60.0 * 24.0 * 30.0),
		"WEEK" | "WEEKS" => current_date() - (number * 60.0 * 60.0 * 24.0 * 7.0),
		"DAY" | "DAYS" => current_date() - (number * 60.0 * 60.0 * 24.0),
		_ => current_date(),
	}
}
