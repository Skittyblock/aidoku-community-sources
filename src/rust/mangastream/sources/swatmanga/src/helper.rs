use aidoku::{
	error::Result,
	prelude::{format, println},
	std::net::{HttpMethod, Request},
	std::{json::parse, String, Vec},
	Manga, MangaContentRating, MangaViewer, Page,
};
use mangastream_template::{
	helper::{append_protocol, manga_status},
	template::MangaStreamSource,
};

// parse manga details page
pub fn parse_manga_details(source: &MangaStreamSource, id: String) -> Result<Manga> {
	let html = Request::new(id.as_str(), HttpMethod::Get).html();
	let title = html.select(source.manga_details_title).text().read();
	let image = html
		.select(source.manga_details_cover)
		.attr("src")
		.read()
		.replace("?resize=165,225", "");
	let cover: String = if image.starts_with("data:") || image.is_empty() {
		let title_id = title.replace(' ', "+").replace('’', "%27");
		let url = format!("{}/?s={}", source.base_url, title_id);
		println!("{} -> {}", title_id, url);
		Request::new(&url, HttpMethod::Get)
			.html()
			.select(".limit img")
			.attr("src")
			.read()
	} else {
		image
	};

	println!("{}", cover);
	let mut author = String::from(
		html.select(source.manga_details_author)
			.text()
			.read()
			.trim(),
	);
	if author == "-" {
		author = String::from("No Author");
	}
	let artist = html.select(source.manga_details_artist).text().read();
	let description = html.select(source.manga_details_description).text().read();
	let status = manga_status(
		String::from(
			html.select(source.manga_details_status)
				.text()
				.read()
				.trim(),
		),
		source.status_options,
		source.status_options_2,
	);
	let mut categories = Vec::new();
	let mut nsfw = if source.is_nsfw {
		MangaContentRating::Nsfw
	} else {
		MangaContentRating::Safe
	};
	for node in html.select(source.manga_details_categories).array() {
		let category = node.as_node().text().read();
		for genre in source.nsfw_genres.iter() {
			if *genre == category {
				nsfw = MangaContentRating::Nsfw
			}
		}
		categories.push(category.clone());
	}
	let manga_type = html.select(source.manga_details_type).text().read();
	let viewer = if manga_type.as_str() == source.manga_details_type_options {
		MangaViewer::Rtl
	} else {
		MangaViewer::Scroll
	};
	Ok(Manga {
		id: id.clone(),
		cover: append_protocol(cover),
		title,
		author,
		artist,
		description,
		url: id,
		categories,
		status,
		nsfw,
		viewer,
	})
}

pub fn parse_page_list(source: &MangaStreamSource, id: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	let html = Request::new(&id, HttpMethod::Get)
        .header("User-Agent","Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/42.0.2311.135 Safari/537.36 Edge/12.246")
        .header("Referer", &source.base_url)
		.html();

	let script = html
		.select("script[data-optimized=1]")
		.attr("data-src")
		.read();
	let raw_text = Request::new(&script, HttpMethod::Get).string();
	let trimmed_json = &raw_text
		[raw_text.find(r#":[{"s"#).unwrap_or(0) + 2..raw_text.rfind("]}],").unwrap_or(0) + 2];
	let json = parse(trimmed_json.as_bytes()).as_object()?;
	let images = json.get("images").as_array()?;
	for (index, page) in images.enumerate() {
		let page_url = page.as_string()?.read();
		println!("{}", page_url);
		pages.push(Page {
			index: index as i32,
			url: page_url,
			base64: String::new(),
			text: String::new(),
		});
	}
	Ok(pages)
}
