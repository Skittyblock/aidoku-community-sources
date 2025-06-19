use aidoku::{
	error::Result,
	prelude::*,
	std::{defaults::defaults_get, net::Request, String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, MangaStatus, MangaViewer, Page,
};

use crate::helper::*;

pub fn parse_manga_list(base_url: String, filters: Vec<Filter>) -> Result<MangaPageResult> {
	let (query, search) = check_for_search(filters);

	let url = {
		if search {
			format!("{}/search?keyword={}", base_url, query)
		} else {
			// This is to handle parse_manga_listing as it passes in full a url,
			// not just the base
			if base_url.contains("genre") {
				base_url
			} else {
				format!("{}/genre", base_url)
			}
		}
	};

	let html = request(&url, false).html()?;

	let mut mangas: Vec<Manga> = Vec::new();

	for manga in html
		.select("#content > div.webtoon_list_wrap ul > li > a")
		.array()
	{
		let manga_node = manga.as_node().expect("Failed to get manga node");
		let url = manga_node.attr("href").read();
		let id = get_manga_id(&url);
		let cover = manga_node.select("img").attr("src").read();
		let title = manga_node.select(".title").text().read();

		mangas.push(Manga {
			id,
			cover,
			title,
			url,
			viewer: MangaViewer::Scroll,
			..Default::default()
		});
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more: false,
	})
}

pub fn parse_canvas_list(url: String, page: i32) -> Result<MangaPageResult> {
	// Canvas series are series uploaded by individual artists,
	// aka unlicensed series
	let canvas_series = defaults_get("canvasSeries")?.as_bool().unwrap_or(true);
	// If canvas series are disabled, return an empty result
	if !canvas_series {
		return Ok(MangaPageResult {
			..Default::default()
		});
	};

	let url = format!("{}&page={}", url, page);

	let html = request(&url, false).html()?;

	let mut mangas: Vec<Manga> = Vec::new();

	for manga in html
		.select("#content div.challenge_lst > ul > li > a")
		.array()
	{
		let manga_node = manga.as_node().expect("Failed to get manga node");
		let url = manga_node.attr("href").read();
		let id = get_manga_id(&url);
		let cover = manga_node.select("img").attr("src").read();
		let title = manga_node.select(".subj").text().read();

		mangas.push(Manga {
			id,
			cover,
			title,
			url,
			viewer: MangaViewer::Scroll,
			..Default::default()
		});
	}

	let has_more =
		html.select("#content > div.cont_box > div.challenge_cont_area > div.paginate > a.pg_next")
			.text()
			.read() != "";

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_manga_listing(
	base_url: String,
	listing: Listing,
	page: i32,
) -> Result<MangaPageResult> {
	let url = {
		match listing.name.as_str() {
			"Latest" => format!("{}/genre?sortOrder=UPDATE", base_url),
			"Popular" => format!("{}/genre?sortOrder=MANA", base_url),
			"Top" => format!("{}/genre?sortOrder=LIKEIT", base_url),
			"Canvas Latest" => format!("{}/canvas/list?genreTab=ALL&sortOrder=UPDATE", base_url),
			"Canvas Popular" => {
				format!("{}/canvas/list?genreTab=ALL&sortOrder=READ_COUNT", base_url)
			}
			"Canvas Top" => format!("{}/canvas/list?genreTab=ALL&sortOrder=LIKEIT", base_url),
			_ => format!("{}/genre", base_url),
		}
	};

	if url.contains("canvas") {
		parse_canvas_list(url, page)
	} else {
		parse_manga_list(url, Vec::new())
	}
}

pub fn parse_manga_details(base_url: String, manga_id: String) -> Result<Manga> {
	let url = get_manga_url(&manga_id, base_url);

	let html = request(&url, false).html()?;

	let cover = html
		.select("head meta[property=\"og:image\"]")
		.first()
		.attr("content")
		.read();

	let title = html
		.select("#content > div.cont_box > div.detail_header > div.info > .subj")
		.text()
		.read();

	let author_artist = html
		.select("#content > div.cont_box > div.detail_header > div.info > .author_area")
		.text()
		.read()
		.replace("author info", "");
	let author_artist = author_artist.split(',').collect::<Vec<&str>>();

	let author = author_artist
		.first()
		.map(|s| s.trim())
		.map(String::from)
		.unwrap_or_default();
	let mut artist = String::new();

	if author_artist.len() > 1 {
		artist = String::from(author_artist[1].trim());
	}

	let description = html.select("#_asideDetail > .summary").text().read();

	let status = {
		let status_text = html
			.select("#_asideDetail > .day_info")
			.text()
			.read()
			.to_lowercase();

		let series_note = html
			.select("#content > div.cont_box > div.detail_body div.detail_paywall")
			.text()
			.read()
			.to_lowercase();

		// Even if a series is on hiatus it will have "every x" in the status text
		// So we have to check the series note for hiatus before checking for ongoing
		if status_text.contains("completed") {
			MangaStatus::Completed
		} else if series_note.contains("will return") {
			MangaStatus::Hiatus
		} else if status_text.contains("every") {
			MangaStatus::Ongoing
		} else {
			MangaStatus::Unknown
		}
	};

	let mut categories: Vec<String> = Vec::new();
	let categories_selector =
		html.select("#content > div.cont_box > div.detail_header > div.info > .genre");

	for category in categories_selector.array() {
		let category_node = category.as_node().expect("Failed to get category node");
		let category = category_node.text().read();
		categories.push(category);
	}

	let url = html.base_uri().read();

	Ok(Manga {
		id: manga_id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories,
		status,
		viewer: MangaViewer::Scroll,
		..Default::default()
	})
}

pub fn parse_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let base_url = get_base_url_no_lang(true);
	let api_url = if let Some(canvas_id) = manga_id.strip_suffix("-canvas") {
		format!(
			"{}/api/v1/canvas/{}/episodes?pageSize=100000",
			base_url, canvas_id
		)
	} else {
		format!(
			"{}/api/v1/webtoon/{}/episodes?pageSize=100000",
			base_url, manga_id
		)
	};

	let json = request(&api_url, true).json()?;
	let episode_list = json
		.as_object()?
		.get("result")
		.as_object()?
		.get("episodeList")
		.as_array()?;

	let lang = get_lang_code().unwrap_or(String::from("en"));

	let mut chapters: Vec<Chapter> = Vec::new();

	for episode in episode_list.rev() {
		let Ok(object) = episode.as_object() else {
			continue;
		};
		let url = format!(
			"{}{}",
			base_url,
			object.get("viewerLink").as_string()?.read()
		);
		let id = get_chapter_id(&url);

		let mut volume = -1.0;

		let title = {
			let raw_title = object
				.get("episodeTitle")
				.as_string()
				.map(|s| s.read())
				.unwrap_or_default();
			let mut title = raw_title.split_whitespace().collect::<Vec<&str>>();

			// Remove leading volume text and set volume accordingly
			// This is for titles like "(S1) Chapter 1 - PeePeePooPoo"
			// or for titles like "(T1) Chapter 1 - PeePeePooPoo"
			if !title.is_empty() {
				let title_chars = title[0].chars().collect::<Vec<char>>();

				if title_chars.len() >= 3
					&& (title_chars[1] == 'S' || title_chars[1] == 'T')
					&& String::from(title_chars[2]).parse::<f64>().is_ok()
				{
					volume = String::from(title_chars[2]).parse::<f32>().unwrap_or(-1.0);
					title.remove(0);
				}

				// Remove leading episode text
				// This is for titles like "Ep.1 - PeePeePooPoo"
				if title_chars.len() >= 4
					&& (title_chars[0] == 'E'
						&& (title_chars[1] == 'p' || title_chars[1] == 'P')
						&& title_chars[2] == '.')
					&& title_chars[3..]
						.iter()
						.collect::<String>()
						.parse::<f64>()
						.is_ok()
				{
					title.remove(0);
				}
			}

			// Remove leading season text and set volume accordingly
			// This is for titles like "[Season 1] Chapter 1 - PeePeePooPoo"
			if title.len() >= 2
				&& (title[0] == "[Season")
				&& title[1].replace(']', "").parse::<f64>().is_ok()
			{
				volume = title[1].replace(']', "").parse::<f32>().unwrap_or(-1.0);
				title.remove(0);
				title.remove(0);
			}

			// Remove leading chapter/episode text
			if title.len() >= 2
				&& (title[0] == "Chapter"
					|| title[0] == "Episode"
					|| title[0] == "Ch."
					|| title[0] == "CH."
					|| title[0] == "Ep."
					|| title[0] == "EP"
					|| title[0] == "EP.")
				&& title[1].replace(':', "").parse::<f64>().is_ok()
			{
				title.remove(0);
				title.remove(0);
			}

			// Remove leading symbols
			if !title.is_empty() && (title[0] == "-" || title[0] == ":") {
				title.remove(0);
			}

			title.join(" ")
		};

		let chapter = object.get("episodeNo").as_float().unwrap_or(-1.0) as f32;
		let date_updated = object
			.get("exposureDateMillis")
			.as_float()
			.map(|f| f / 1000.0)
			.unwrap_or(-1.0);

		chapters.push(Chapter {
			id,
			title,
			volume,
			chapter,
			date_updated,
			url,
			lang: lang.clone(),
			..Default::default()
		});
	}

	Ok(chapters)
}

pub fn parse_page_list(
	base_url: String,
	manga_id: String,
	chapter_id: String,
) -> Result<Vec<Page>> {
	let url = get_chapter_url(chapter_id, manga_id, base_url);

	let html = request(&url, false).html()?;

	let mut pages: Vec<Page> = Vec::new();

	// Optional pages contain "?type=opti" at the end of the url
	let optional_pages = defaults_get("optionalPages")?.as_bool().unwrap_or(true);

	for (index, page) in html.select("div#_imageList > img").array().enumerate() {
		let page_node = page.as_node().expect("Failed to get page node");

		let url = page_node.attr("data-url").read();

		// Skip optional pages if optionalPages is false
		if url.ends_with("?type=opti") && !optional_pages {
			continue;
		}

		pages.push(Page {
			index: index as i32,
			url,
			..Default::default()
		});
	}

	Ok(pages)
}

pub fn modify_image_request(base_url: String, request: Request) {
	request.header("Referer", &base_url);
}

pub fn handle_url(base_url: String, url: String) -> Result<DeepLink> {
	let manga_id = get_manga_id(&url);
	let parsed = parse_manga_details(base_url, manga_id);

	Ok(DeepLink {
		manga: parsed.ok(),
		..Default::default()
	})
}
