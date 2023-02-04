use aidoku::{
	error::Result,
	helpers::substring::Substring,
	prelude::*,
	std::{
		net::{HttpMethod, Request},
		String, StringRef, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

use crate::helper::{append_domain, get_chapter_number, get_search_url, manga_status};

pub fn parse_manga_list(
	base_url: String,
	filters: Vec<Filter>,
	page: i32,
) -> Result<MangaPageResult> {
	let mut included_tags: Vec<String> = Vec::new();
	let mut title: String = String::new();
	let status_options = ["", "ativo", "completo"];
	let sort_options = [
		"", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q",
		"R", "S", "T", "U", "V", "W", "X", "Y", "Z",
	];
	let mut sort_letter: String = String::new();
	let mut status: String = String::new();
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				title = filter.value.as_string()?.read();
			}
			FilterType::Genre => match filter.value.as_int().unwrap_or(-1) {
				1 => included_tags.push(filter.object.get("id").as_string()?.read()),
				_ => continue,
			},

			FilterType::Select => {
				let index = filter.value.as_int().unwrap_or(-1) as usize;
				match filter.name.as_str() {
					"Status" => status = String::from(status_options[index]),
					"ByLetter" => sort_letter = String::from(sort_options[index]),
					_ => continue,
				}
			}
			_ => continue,
		};
	}
	let url = get_search_url(base_url, title, page, included_tags, status, sort_letter);
	parse_manga_listing(url, String::new(), page)
}

pub fn parse_manga_listing(
	base_url: String,
	listing_name: String,
	page: i32,
) -> Result<MangaPageResult> {
	let list_url = if !listing_name.is_empty() {
		match listing_name.as_str() {
			"Últimas Atualizações" => format!("{base_url}/index.php?pagina={page}"),
			_ => format!("{base_url}/mangabr&pagina={page}"),
		}
	} else {
		base_url.clone()
	};
	let mut _count = 0;
	let mut mangas: Vec<Manga> = Vec::new();
	let html = Request::new(&list_url, HttpMethod::Get).html()?;
	if listing_name.is_empty() {
		for manga in html.select(".mangas a").array() {
			let manga_node = manga.as_node().expect("Failed to get manga as node");
			let title = manga_node.select("h3").text().read();
			let url_path = manga_node.attr("href").read();
			let url = append_domain(base_url.clone(), url_path.clone());
			let id = String::from(url_path.substring_after("mangabr/").unwrap_or_default());
			let cover = append_domain(
				String::from("https://goldenmangas.top"),
				manga_node
					.select("img")
					.attr("src")
					.read()
					.replace(' ', "%20"),
			);
			let viewer = if title.contains("Novel") {
				MangaViewer::Ltr
			} else {
				MangaViewer::Rtl
			};
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
				viewer,
			});
		}
	} else {
		for manga in html.select("#response .row").array() {
			let manga_node = manga.as_node().expect("Failed to get manga as node");
			let title = manga_node.select("div > a h3").text().read();
			let url_path = manga_node.select("div > a").attr("href").read();
			let url = append_domain(base_url.clone(), url_path.clone());
			let id = String::from(url_path.substring_after("mangabr/").unwrap_or_default());
			let cover = append_domain(
				base_url.clone(),
				manga_node
					.select("img")
					.attr("src")
					.read()
					.replace(' ', "%20"),
			);
			let viewer = if title.contains("Novel") {
				MangaViewer::Ltr
			} else {
				MangaViewer::Rtl
			};
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
				viewer,
			});
		}
	}
	let has_more = (html.select(".pagination li").text().read().trim()).contains('»');

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_manga_details(base_url: String, id: String) -> Result<Manga> {
	let url = format!("{base_url}/mangabr/{id}");
	let html = Request::new(&url, HttpMethod::Get).html()?;
	let title = html
		.select("div.row > div.col-sm-8 > h2:nth-child(1)")
		.text()
		.read();
	let cover = html.select(".single-comic .thumb img").attr("src").read();
	let author = html
		.select("a[href*=autor]")
		.array()
		.map(|val| val.as_node().expect("Failed to get author").text().read())
		.collect::<Vec<String>>()
		.join(", ");
	let artist = html
		.select("a[href*=artista]")
		.array()
		.map(|val| val.as_node().expect("Failed to get author").text().read())
		.collect::<Vec<String>>()
		.join(", ");
	let description = html.select("#manga_capitulo_descricao > p").text().read();
	let categories = html
		.select("div.container.manga h5:nth-child(3) > a")
		.array()
		.map(|val| val.as_node().expect("Failed to get author").text().read())
		.collect::<Vec<String>>();
	let status = manga_status(html.select("a[href*=status]").first().text().read());
	let nsfw = if categories.iter().any(|v| {
		*v == "Adulto (18+)"
			|| *v == "Adulto (YAOI)"
			|| *v == "Hentai"
			|| *v == "Ecchi"
			|| *v == "Smut"
	}) {
		MangaContentRating::Nsfw
	} else {
		MangaContentRating::Safe
	};
	let viewer = if title.contains("Novel") {
		MangaViewer::Ltr
	} else {
		MangaViewer::Rtl
	};
	Ok(Manga {
		id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories: categories
			.into_iter()
			.filter(|i| !i.is_empty())
			.collect::<Vec<String>>(),
		status,
		nsfw,
		viewer,
	})
}

pub fn parse_chapter_list(base_url: String, id: String) -> Result<Vec<Chapter>> {
	let url = format!("{base_url}/mangabr/{}", id);
	let mut chapters: Vec<Chapter> = Vec::new();
	let html = Request::new(&url, HttpMethod::Get)
		.header("referer", &url)
		.html()?;
	for chapter in html.select(".capitulos li").array() {
		let chapter_node = chapter.as_node().expect("Failed to get chapter as node");
		let text = chapter_node.select("a").text().read();
		let title = String::from(text.substring_before(" (").unwrap_or_default());
		let chapter_number = get_chapter_number(title.clone());
		let chapter_url = append_domain(
			base_url.clone(),
			chapter_node.select("a").attr("href").read(),
		);
		let chapter_id = String::from(
			chapter_url
				.clone()
				.substring_after_last("/")
				.unwrap_or_default(),
		);
		let scanlator = String::from(text.substring_after_last(") ").unwrap_or_default());
		let date = String::from(
			chapter_node
				.select("span")
				.text()
				.read()
				.substring_after("(")
				.unwrap_or_default()
				.substring_before(")")
				.unwrap_or_default(),
		);
		let date_updated = StringRef::from(date).as_date("dd/MM/yyyy", Some("en_US"), None);
		chapters.push(Chapter {
			id: chapter_id,
			title,
			volume: -1.0,
			chapter: chapter_number,
			date_updated,
			scanlator,
			url: chapter_url,
			lang: String::from("it"),
		});
	}
	Ok(chapters)
}

pub fn parse_page_list(
	base_url: String,
	manga_id: String,
	chapter_id: String,
) -> Result<Vec<Page>> {
	let url = format!("{base_url}/mangabr/{manga_id}/{chapter_id}");
	let mut pages: Vec<Page> = Vec::new();
	let html = Request::new(&url, HttpMethod::Get).html()?;
	for (at, page) in html
		.select("center")
		.first()
		.select("img")
		.array()
		.enumerate()
	{
		let page_node = page.as_node().expect("Failed to get page as node");
		let page_url = append_domain(base_url.clone(), page_node.attr("src").read());
		pages.push(Page {
			index: at as i32,
			url: page_url,
			base64: String::new(),
			text: String::new(),
		});
	}
	Ok(pages)
}

pub fn modify_image_request(base_url: String, request: Request) {
	request.header("Referer", &base_url);
}

pub fn handle_url(base_url: String, url: String) -> Result<DeepLink> {
	let id = String::from(url.substring_after("manga/").unwrap_or_default());
	Ok(DeepLink {
		manga: parse_manga_details(base_url, id).ok(),
		chapter: None,
	})
}
