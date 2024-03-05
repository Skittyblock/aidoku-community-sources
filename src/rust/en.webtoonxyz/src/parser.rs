use aidoku::{
	error::Result, prelude::format, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Chapter, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page, std::html::Node,
};

const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_1_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";

pub fn parse_manga_list(base_url: String, filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	
	let mut search_query = String::new();
	let mut genre = String::new();
	
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				search_query = filter.value.as_string()?.read();
			}
			FilterType::Select => match filter.name.as_str() {
				"Genre" => {
					genre = match filter.value.as_int().unwrap_or(-1) {
						0 => continue,
						1 => String::from("action"),
						2 => String::from("adventure"),
						3 => String::from("bl"),
						4 => String::from("comedy"),
						5 => String::from("drama"),
						6 => String::from("ecchi"),
						7 => String::from("fantasy"),
						8 => String::from("gl"),
						9 => String::from("harem"),
						10 => String::from("historical"),
						11 => String::from("horror"),
						12 => String::from("josei"),
						13 => String::from("martial-arts"),
						14 => String::from("mature"),
						15 => String::from("mystery"),
						16 => String::from("psychological"),
						17 => String::from("romance"),
						18 => String::from("school-life"),
						19 => String::from("sci-fi"),
						20 => String::from("shoujo"),
						21 => String::from("slice-of-life"),
						22 => String::from("smut"),
						23 => String::from("sports"),
						24 => String::from("supernatural"),
						25 => String::from("thriller"),
						_ => continue,
					};
				}
				_ => continue,
			},
			_ => continue,
		}
	}

	let mut url = format!("{}/webtoons/page/{}", base_url, page);
		
	if !search_query.is_empty() {
		url = format!("{}/page/{}/?s={}&post_type=wp-manga", base_url, page, search_query)
	}
	else if !genre.is_empty() {
		url = format!("{}/webtoon-genre/{}/page/{}", base_url, genre, page)
	}

	let (manga_selector, thumb_selector, summary_selector, title_selector) = if !search_query.is_empty() {
		(".c-tabs-item__content", ".tab-thumb", ".tab-summary", ".post-title")
	} else {
		(".manga", ".item-thumb", ".item-summary", ".font-title")
	};

	let html = Request::new(url, HttpMethod::Get).header("User-Agent", USER_AGENT).html()?;
	let manga = parse_manga(&html, manga_selector, thumb_selector, summary_selector, title_selector)?;
	let has_more = !manga.is_empty();

	Ok(MangaPageResult {
		manga,
		has_more,
	})
}

pub fn parse_manga_listing(base_url: String, listing: Listing, page: i32) -> Result<MangaPageResult> {
	let list_query = match listing.name.as_str() {
		"Latest" => "latest",
		"Most Views" => "views",
		"A-Z" => "alphabet",
		"New" => "new-manga",
		"No 18+" => "&adult=0&m_orderby=latest",
		"Only 18+" => "&adult=1&m_orderby=latest",
		_ => "",
	};

	let mut url = format!("{}/webtoons/page/{}/?m_orderby={}", base_url, page, list_query);

	if list_query.contains("adult") {
		url = format!("{}/page/{}/?s&post_type=wp-manga{}", base_url, page, list_query);
	}

	let (manga_selector, thumb_selector, summary_selector, title_selector) = if list_query.contains("adult") {
		(".c-tabs-item__content", ".tab-thumb", ".tab-summary", ".post-title")
	} else {
		(".manga", ".item-thumb", ".item-summary", ".font-title")
	};

	let html = Request::new(url, HttpMethod::Get).header("User-Agent", USER_AGENT).html()?;
	let manga = parse_manga(&html, manga_selector, thumb_selector, summary_selector, title_selector)?;
	let has_more = !manga.is_empty();

	Ok(MangaPageResult {
		manga,
		has_more,
	})
}

pub fn parse_manga_details(base_url: String, manga_id: String) -> Result<Manga> {
	
	let url = format!("{}/read/{}", base_url, manga_id);
	let html = Request::new(&url, HttpMethod::Get).header("User-Agent", USER_AGENT).html()?;
	
	let manga = html.select(".container");
	
	let id = manga_id;
	let cover = manga.select(".summary_image > a > img").attr("data-src").read();
	let title = manga.select(".post-title > h1").text().read();

	let info = manga.select(".post-content_item").array();

	let author = info.get(2).as_node()?.select(".author-content").text().read();
	let artist = info.get(3).as_node()?.select(".artist-content").text().read();
	let description = manga.select(".summary__content").text().read();

	let categories = manga.select(".genres-content > a")
		.array()
		.map(|x| {
			x.as_node().expect("").text().read()
		})
		.collect::<Vec<String>>();

	let mut nsfw_content = MangaContentRating::Safe;
	if manga.select(".post-title").text().read().contains("18+") {
		nsfw_content = MangaContentRating::Nsfw;
	}

	let status_str = info.get(7).as_node()?.select(".summary-content").text().read();
	let status = if status_str.contains("OnGoing") {
		MangaStatus::Ongoing
	} else if status_str.contains("Completed") {
		MangaStatus::Completed
	} else {
		MangaStatus::Unknown
	};

	Ok(Manga {
		id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories,
		status,
		nsfw: nsfw_content,
		viewer: MangaViewer::Scroll,
	})
}

pub fn parse_chapter_list(base_url: String, manga_id: String) -> Result<Vec<Chapter>> {

	let url = format!("{}/read/{}", base_url, manga_id);
	let html = Request::new(url.clone(), HttpMethod::Get).header("User-Agent", USER_AGENT).html()?;
	
	let mut all_chapters: Vec<Chapter> = Vec::new();
	
	for chapter in html.select(".wp-manga-chapter").array() {
		let chapter = chapter.as_node()?;

		let url = chapter.select("a").attr("href").read();
		
		let id = url.split('/').collect::<Vec<&str>>();
		let id = String::from(id[5]);
		
		let index = id.split('-').collect::<Vec<&str>>();
		let index = String::from(index[1]).parse::<f32>().unwrap_or(-1.0);

		//let date_updated = chapter.select(".chapter-release-date").text().as_date("dd MMM YYYY", Some("en_US"), None);

		all_chapters.push(Chapter {
			id,
			chapter: index,
			url,
			..Default::default()
		});
	}

	Ok(all_chapters)
}

pub fn parse_page_list(base_url: String, manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	
	let url = format!("{}/read/{}/{}", base_url, manga_id, chapter_id);
	let html = Request::new(url.clone(), HttpMethod::Get).header("User-Agent", USER_AGENT).html()?;
	
	let mut page_list: Vec<Page> = Vec::new();

	for (i, page) in html.select(".page-break").array().enumerate() {
		let page = page.as_node()?;
		let mut url = page.select("img").attr("data-src").read();
		url = String::from(url.trim());

		page_list.push(Page {
			index: i as i32,
			url,
			..Default::default()
		});
	}

	Ok(page_list)
}

pub fn modify_image_request(base_url: String, request: Request) {
	request
	.header("User-Agent", USER_AGENT)
	.header("Referer", &base_url);
}

fn parse_manga(html: &Node, html_selector: &str, cover_selector: &str, info_selector: &str, title_selector: &str) -> Result<Vec<Manga>> {
	
	let mut mangas: Vec<Manga> = Vec::new();

	for manga in html.select(html_selector).array() {
		let manga = manga.as_node()?;
		
		let cover = manga.select(cover_selector).select("a > img").attr("data-srcset").read();
		let cover = cover.split(", ").collect::<Vec<&str>>();
		let cover = String::from(cover[cover.len() - 1]);
		let cover = cover.split(' ').collect::<Vec<&str>>();
		let cover = String::from(cover[0]);

		let info = manga.select(info_selector);

		let title = info.select(title_selector).text().read();
		let url = manga.select("a").attr("href").read();
		let id = url.split('/').collect::<Vec<&str>>();
		let id = String::from(id[4]);

		mangas.push(Manga {
			id,
			cover,
			title,
			url,
			..Default::default()
		});
	}

	Ok(mangas)
}