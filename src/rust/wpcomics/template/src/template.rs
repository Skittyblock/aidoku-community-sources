use aidoku::{
	error::Result, std::net::HttpMethod, std::net::Request, std::String, std::Vec, Chapter,
	DeepLink, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page,
};

use crate::helper::{append_protocol, extract_f32_from_string, https_upgrade, i32_to_string};

pub struct Selectors {
	pub next_page: &'static str,
	pub manga_cell: &'static str,
	pub manga_cell_url: &'static str,
	pub manga_cell_title: &'static str,
	pub manga_cell_image: &'static str,

	pub manga_details_title: &'static str,
	pub manga_details_title_transformer: fn(String) -> String,
	pub manga_details_cover: &'static str,
	pub manga_details_author: &'static str,
	pub manga_details_author_transformer: fn(String) -> String,
	pub manga_details_description: &'static str,
	pub manga_details_tags: &'static str,
	pub manga_details_tags_splitter: &'static str,
	pub manga_details_status: &'static str,
	pub manga_details_status_transformer: fn(String) -> String,
	pub manga_details_chapters: &'static str,

	pub manga_viewer_page: &'static str,

	pub chapter_date_selector: &'static str,
	pub chapter_anchor_selector: &'static str,
}

pub fn get_manga_list(search_url: String, selectors: &Selectors) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	let mut has_next_page = selectors.next_page.len() > 0;
	let html = Request::new(&search_url, HttpMethod::Get).html();
	for item in html.select(selectors.manga_cell).array() {
		let item_node = item.as_node();
		let title = item_node
			.select(selectors.manga_cell_title)
			.first()
			.text()
			.read();
		let id = https_upgrade(
			item_node
				.select(selectors.manga_cell_url)
				.first()
				.attr("href")
				.read(),
		);
		let cover = https_upgrade(append_protocol(
			item_node
				.select(selectors.manga_cell_image)
				.first()
				.attr("data-original")
				.read(),
		));
		mangas.push(Manga {
			id,
			cover,
			title: (selectors.manga_details_title_transformer)(title),
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: String::new(),
			categories: Vec::new(),
			status: MangaStatus::Unknown,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Default,
		});
	}
	if selectors.next_page.len() > 0 {
		has_next_page = html.select(selectors.next_page).array().len() > 0;
	}
	Ok(MangaPageResult {
		manga: mangas,
		has_more: has_next_page,
	})
}

pub fn get_manga_listing(
	base_url: String,
	listing: Listing,
	selectors: &Selectors,
	listing_mapping: fn(String) -> String,
	page: i32,
) -> Result<MangaPageResult> {
	let mut url = String::new();
	url.push_str(&base_url);
	if listing.name != String::from("All") {
		url.push_str("/");
		url.push_str(&listing_mapping(listing.name));
		url.push_str("?page=");
		url.push_str(&i32_to_string(page));
	} else {
		url.push_str("/?page=");
		url.push_str(&i32_to_string(page));
	}
	get_manga_list(url, selectors)
}

pub fn get_manga_details(
	id: String,
	selectors: &Selectors,
	default_viewer: MangaViewer,
	status_from_string: fn(String) -> MangaStatus,
) -> Result<Manga> {
	let details = Request::new(id.clone().as_str(), HttpMethod::Get).html();
	let title = details.select(selectors.manga_details_title).text().read();
	let cover = append_protocol(
		details
			.select(selectors.manga_details_cover)
			.attr("src")
			.read(),
	);
	let author = (selectors.manga_details_author_transformer)(
		details.select(selectors.manga_details_author).text().read(),
	);
	let description = details
		.select(selectors.manga_details_description)
		.text()
		.read();
	let mut categories = Vec::new();
	let mut nsfw = MangaContentRating::Safe;
	let mut viewer = default_viewer;

	if selectors.manga_details_tags.len() > 0 {
		for node in details
			.select(selectors.manga_details_tags)
			.text()
			.read()
			.split(selectors.manga_details_tags_splitter)
		{
			let category = String::from(node);
			if category == String::from("Smut")
				|| category == String::from("Mature")
				|| category == String::from("Adult")
				|| category == String::from("18+")
			{
				nsfw = MangaContentRating::Nsfw;
			} else if category == String::from("Ecchi") || category == String::from("16+") {
				nsfw = MangaContentRating::Suggestive;
			}
			if category.contains("Webtoon")
				|| category.contains("Manhwa")
				|| category.contains("Manhua")
			{
				viewer = MangaViewer::Scroll;
			}
			categories.push(category.clone());
		}
	}
	let status = status_from_string((selectors.manga_details_status_transformer)(
		details.select(selectors.manga_details_status).text().read(),
	));
	Ok(Manga {
		id: id.clone(),
		cover,
		title: (selectors.manga_details_title_transformer)(title),
		author,
		artist: String::new(),
		description,
		url: id.clone(),
		categories,
		status,
		nsfw,
		viewer,
	})
}

pub fn get_chapter_list(
	id: String,
	selectors: &Selectors,
	skip_first: bool,
	chapter_date_converter: fn(String) -> f64,
) -> Result<Vec<Chapter>> {
	let mut skipped_first = false;
	let mut chapters: Vec<Chapter> = Vec::new();
	let html = Request::new(id.clone().as_str(), HttpMethod::Get).html();
	let title_untrimmed = (selectors.manga_details_title_transformer)(
		html.select(selectors.manga_details_title).text().read(),
	);
	let title = title_untrimmed.trim();
	for chapter in html.select(selectors.manga_details_chapters).array() {
		if skip_first && !skipped_first {
			skipped_first = true;
			continue;
		}
		let chapter_node = chapter.as_node();
		let chapter_url = chapter_node
			.select(selectors.chapter_anchor_selector)
			.attr("href")
			.read()
			.replacen("http://", "https://", 1);
		let chapter_id = chapter_url.clone();
		let chapter_title = chapter_node
			.select(selectors.chapter_anchor_selector)
			.text()
			.read();
		let chapter_number =
			extract_f32_from_string(String::from(title), String::from(&chapter_title));
		let date_updated = chapter_date_converter(
			chapter_node
				.select(selectors.chapter_date_selector)
				.text()
				.read(),
		);
		chapters.push(Chapter {
			id: chapter_id,
			title: chapter_title,
			volume: -1.0,
			chapter: chapter_number,
			date_updated,
			scanlator: String::new(),
			url: chapter_url,
			lang: String::from("en"),
		});
	}
	Ok(chapters)
}

pub fn get_page_list(
	id: String,
	selectors: &Selectors,
	all_pages_reader_suffix: String,
	url_transformer: fn(String) -> String,
) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	let mut url = id.clone();
	url.push_str(&all_pages_reader_suffix);
	let html = Request::new(&url, HttpMethod::Get).html();
	let mut at = 0;
	for page in html.select(selectors.manga_viewer_page).array() {
		let page_node = page.as_node();
		let mut page_url = page_node.attr("data-original").read();
		if !page_url.starts_with("http") {
			page_url = String::from(String::from("https:") + &page_url);
		}
		pages.push(Page {
			index: at,
			url: url_transformer(page_url),
			base64: String::new(),
			text: String::new(),
		});
		at += 1;
	}
	Ok(pages)
}

pub fn modify_image_request(base_url: String, user_agent: String, request: Request) {
	request
		.header("Referer", &base_url)
		.header("User-Agent", &user_agent);
}

pub fn handle_url(
	url: String,
	selectors: &Selectors,
	default_viewer: MangaViewer,
	status_from_string: fn(String) -> MangaStatus,
) -> Result<DeepLink> {
	Ok(DeepLink {
		manga: Some(get_manga_details(
			url.clone(),
			selectors,
			default_viewer,
			status_from_string,
		)?),
		chapter: None,
	})
}
