use aidoku::{
	prelude::*,
	error::Result,
	std::{
		html::Node,
		String, StringRef, Vec, current_date, json
	},
	Manga, Page, MangaPageResult, MangaStatus, MangaContentRating, MangaViewer, Chapter
};

pub fn parse_manga_list(html: Node) -> Result<MangaPageResult>  {
	let mut mangas: Vec<Manga> = Vec::new();

	for page in html.select(".group").array() {
		let page = page
			.as_node()
			.expect("Failed to get data as array of nodes");
		
		let title = page.select(">.title a").attr("title").read();
		let url = page.select(">.title a").attr("href").read();
		let id = String::from(url.split('/').enumerate().nth(4).expect("Failed to get id").1.trim());
		let cover = String::from(page.select(".preview").attr("src").read().trim());

		mangas.push(Manga {
			id,
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url,
			categories: Vec::new(),
			status: MangaStatus::Unknown,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Rtl
		});
	}

	let has_more = !html.select(".next").text().read().is_empty();

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_manga_details(base_url: String, manga_id: String, html: Node) -> Result<Manga> {	
	let cover = String::from(html.select(".thumbnail img").attr("src").read().trim());
	let title = html.select(".large.comic .title").text().read();
	let mut author = String::new();
	let mut artist = String::new();
	let mut description = String::new();
	let url = format!("{}/series/{}", base_url, manga_id);
	
	for item in html.select(".large.comic .info").html().read().split("<br>"){
		let split :Vec<&str> = item.trim().split(':').collect();

		match split[0].trim() {
			"<b>Author</b>" => author = String::from(split[1].trim()),
			"<b>Artist</b>" => artist = String::from(split[1].trim()),
			"<b>Synopsis</b>" => description = String::from(split[1].trim()),
			_ => ()
		}
	}

	Ok(Manga {
		id: manga_id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories: Vec::new(),
		status: MangaStatus::Unknown,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Scroll
	})
}

pub fn parse_chapter_list(base_url: String, manga_id: String, html: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	for element in html.select(".list .element").array() {
		let element = element
			.as_node()
			.expect("Failed to get data as array of nodes");

		let url = element.select(".title a").attr("href").read();
		let id = String::from(&url.replace(&format!("{}/read/{}", base_url, manga_id), ""));

		let split_url : Vec<&str> = url.split('/').collect();
		let volume = if split_url[6] == "0" {
			-1.0
		} else {
			String::from(split_url[6]).parse().unwrap()
		} as f32;

		let chapter = if split_url[8].is_empty() {
			String::from(split_url[7]).parse().unwrap()
		} else {
			format!("{}.{}", split_url[7], split_url[8]).parse().unwrap()
		};

		let chap_title_str = element.select(".title a").text().read();
		let mut title = String::new();
		if chap_title_str.contains(':') {
			let split_title :Vec<&str> = chap_title_str.split(':').collect();
			title = String::from(split_title[1].trim());
		}
		
		let date_str = element.select(".meta_r").text().read();
		let date_str_split :Vec<&str> = date_str.split(',').collect();
		let scanlator = String::from(date_str_split[0].replace("par", "").trim());

		let mut date_updated = StringRef::from(&date_str_split[1].trim())
			.0
			.as_date("YYYY.MM.d", Some("fr"), None)
			.unwrap_or(-1.0);

		if date_updated < -1.0 {
			date_updated = StringRef::from(&date_str)
				.0
				.as_date("YYYY.MM.d", Some("fr"), None)
				.unwrap_or(-1.0);
		}
		if date_updated == -1.0 {
			date_updated = current_date();
		}

		chapters.push(Chapter{
			id,
			title,
			volume,
			chapter,
			date_updated,
			scanlator,
			url,
			lang: String::from("fr"),
		});
	}

	Ok(chapters)
}

pub fn parse_page_list(html: Node) -> Result<Vec<Page>> {
	let data = html
		.select("#content > script:nth-child(5)")
		.html()
		.read()
		.lines()
		.enumerate()
		.nth(1)
		.expect("Failed to get str 'var pages'")
		.1
		.trim()
		.replace("var pages =", "")
		.replace("];", "]");

	let json = json::parse(data)
		.unwrap()
		.as_array()
		.expect("Failed to get data as array");
	
	let mut pages: Vec<Page> = Vec::new();
	for (i, item) in json.enumerate() {
		let obj = item
			.as_object()
			.expect("Failed to get data as object");

		let url = obj
			.get("url")
			.as_string()
			.expect("Failed to get data cover as string")
			.read();

		pages.push(Page {
			index: i as i32,
			url,
			base64: String::new(),
			text: String::new(),
		});
	}

	Ok(pages)
}
