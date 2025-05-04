#![no_std]

mod helper;

use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	prelude::*,
	std::html::Node,
	std::net::Request,
	std::{net::HttpMethod, String, Vec},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};
use core::cmp::Ordering;

fn parse_user_hash_and_query(document: &Node) -> Result<(String, String)> {
	let mut script_data = String::new();
	for script in document.select("script").array() {
		let node = script.as_node().expect("Script not a node");
		let text = node.outer_html().read();
		if text.contains("site_login_hash") {
			script_data = text;
			break;
		}
	}

	let user_hash = script_data
		.split("site_login_hash = '")
		.nth(1)
		.and_then(|s| s.split('\'').next())
		.ok_or(AidokuError {
			reason: AidokuErrorKind::NodeError(aidoku::error::NodeError::ParseError),
		})?;

	let hash_query = "user_hash";

	Ok((String::from(hash_query), String::from(user_hash)))
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut manga_arr: Vec<Manga> = Vec::new();
	let mut total: i32 = 1;

	let base_url = String::from("https://manga.in.ua");

	let genres_list = helper::genres_list();

	let mut sort_value: String = String::new();
	let mut search_value = String::new();
	let mut genre_value = String::new();
	let mut status_value: String = String::new();

	let mut is_cover_data_src = false;

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				search_value = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				if filter.name.as_str() == "Сортувати" {
					let index = filter.value.as_int()? as usize;

					let option = match index {
						0 => "latest",
						1 => "popular",
						_ => "",
					};
					sort_value = String::from(option);
				}

				if filter.name.as_str() == "Жанри" {
					let index = filter.value.as_int()? as usize;
					match index {
						0 => continue,
						_ => genre_value = String::from(genres_list[index]),
					}
				}

				if filter.name.as_str() == "Статус перекладу" {
					let index = filter.value.as_int()? as usize;

					let option = match index {
						0 => continue,
						1 => "xfsearch/tra/%D0%97%D0%B0%D0%BA%D1%96%D0%BD%D1%87%D0%B5%D0%BD%D0%B8%D0%B9", // Закінчений - Completed
						2 => "xfsearch/tra/%D0%A2%D1%80%D0%B8%D0%B2%D0%B0%D1%94", // Триває - Ongoing
						3 => "xfsearch/tra/%D0%9D%D0%B5%D0%B2%D1%96%D0%B4%D0%BE%D0%BC%D0%BE", // Невідомо - Unknown 
						4 => "xfsearch/tra/%D0%9F%D0%BE%D0%BA%D0%B8%D0%BD%D1%83%D1%82%D0%BE", // Покинуто - Cancelled
						_ => continue
					};
					status_value = String::from(option);
				}
			}
			_ => todo!(),
		}
	}

	// in case the user searches on popular - move to latest because popular doesn't
	// have a search
	if sort_value == "popular"
		&& (!search_value.is_empty() || !genre_value.is_empty() || !status_value.is_empty())
	{
		sort_value = String::from("latest");
	}

	if sort_value == "popular" {
		// ignore page number
		let html = Request::new("https://manga.in.ua/", HttpMethod::Get)
			.html()
			.expect("");

		for result in html.select(".owl-carousel .card--big").array() {
			let res_node = result
				.as_node()
				.expect("popular html array not an array of nodes");

			let href = res_node.select("a").attr("href").read();

			let title = res_node
				.select(".card__content .card__title a")
				.text()
				.read();
			let cover = res_node
				.select(".card__cover a figure img")
				.attr("abs:src")
				.read();

			let mut is_nsfw = false;
			let mut categories: Vec<String> = Vec::new();
			for categ_res in res_node.select(".card__category a").array() {
				let categ_node = categ_res
					.as_node()
					.expect("html array not an array of nodes");

				let name = categ_node.text().read();
				is_nsfw = helper::is_nsfw(&name);

				categories.push(name);
			}

			manga_arr.push(Manga {
				id: href,
				cover,
				title,
				categories,
				nsfw: if is_nsfw {
					MangaContentRating::Nsfw
				} else {
					MangaContentRating::Safe
				},
				viewer: MangaViewer::Rtl,
				..Default::default()
			});
		}
	} else {
		let request;

		// have search, genre and status -> ignore status
		if !search_value.is_empty() && !genre_value.is_empty() {
			//&& !status_value.is_empty(){
			let url = format!("{}/mangas/{}", base_url, genre_value);

			let body_data = format!(
				"do=search&subaction=search&titleonly=3&story={}",
				search_value
			);

			request = Request::new(url.as_str(), HttpMethod::Post)
				.body(body_data.as_bytes())
				.header("Referer", "https://manga.in.ua");
		}
		// search and status
		else if !search_value.is_empty() && genre_value.is_empty() && !status_value.is_empty() {
			let url = format!("{}/{}", base_url, status_value);

			let body_data = format!(
				"do=search&subaction=search&titleonly=3&story={}",
				search_value
			);

			request = Request::new(url.as_str(), HttpMethod::Post)
				.body(body_data.as_bytes())
				.header("Referer", "https://manga.in.ua");
		}
		// genre and status -> ignore status
		else if search_value.is_empty() && !genre_value.is_empty() && !status_value.is_empty() {
			let url = format!("{}/mangas/{}/page/{}", base_url, genre_value, page);
			request = Request::new(url.as_str(), HttpMethod::Get);

			is_cover_data_src = true;
		}
		// only search
		else if !search_value.is_empty() && genre_value.is_empty() && status_value.is_empty() {
			let url = format!("{}/index.php?do=search", base_url);

			let body_data = format!(
				"do=search&subaction=search&story={}&search_start={}",
				search_value, page
			);

			request = Request::new(url.as_str(), HttpMethod::Post)
				.body(body_data.as_bytes())
				.header("Referer", "https://manga.in.ua");
		}
		// only genre
		else if search_value.is_empty() && !genre_value.is_empty() && status_value.is_empty() {
			let url = format!("{}/mangas/{}/page/{}", base_url, genre_value, page);
			request = Request::new(url.as_str(), HttpMethod::Get);

			is_cover_data_src = true;
		}
		// only status
		else if search_value.is_empty() && genre_value.is_empty() && !status_value.is_empty() {
			let url = format!("{}/{}/page/{}", base_url, status_value, page);
			request = Request::new(url.as_str(), HttpMethod::Get);

			is_cover_data_src = true;
		}
		// default case
		else {
			let url = format!("{}/page/{}", base_url, page);
			request = Request::new(url.as_str(), HttpMethod::Get);

			is_cover_data_src = true;
		}

		let html = request
			.html()
			.expect("latest html array not an array of nodes");

		for result in html.select(".main .item").array() {
			let res_node = result.as_node().expect("");

			let card = res_node.select(".card--big .card__content");

			let title = card.select(".card__title a").text().read();

			let href = card.select(".card__title a").attr("href").read();

			let mut cover: String = if is_cover_data_src {
				res_node
					.select(".card--big img")
					.attr("abs:data-src")
					.read()
			} else {
				res_node.select(".card--big img").attr("abs:src").read()
			};

			cover = cover.replace("//", "/");

			let mut categories: Vec<String> = Vec::new();
			for categ_res in res_node.select(".card__category a").array() {
				let categ_node = categ_res.as_node().expect("");
				let categ_name = categ_node.text().read();

				categories.push(categ_name);
			}

			let mut desc = String::new();
			let mut is_nsfw = false;
			for info in card.select(".card__list li").array() {
				let info_node = info.as_node().expect("");
				desc = info_node.text().read();

				if desc.contains('+') {
					is_nsfw = helper::is_nsfw(&desc);
				}
			}

			manga_arr.push(Manga {
				id: href,
				cover,
				title,
				description: desc,
				categories,
				nsfw: if is_nsfw {
					MangaContentRating::Nsfw
				} else {
					MangaContentRating::Safe
				},
				viewer: MangaViewer::Rtl,
				..Default::default()
			})
		}

		let mut last_page = String::new();
		for paging_res in html.select(".page-navigation a").array() {
			let paging = paging_res.as_node().expect("");
			let st = paging.text().read();

			// first page  -> 	<a>1</1a> ... <a>99</a> <span>Попередня</span>
			// <a>Наступна</a> other pages ->	<a>1</1a> ... <a>99</a> <a>Попередня</a>
			// <a>Наступна</a> on first page "Попередня" (previous) is span, so the
			// latest page number is before "Наступна" (Next) on other pages "Попередня"
			// and "Наступна" are a elements, so latest page number is before "Попередня"
			if st != "Наступна" && st != "Попередня" {
				last_page = st.clone();
			}
		}

		match last_page.parse::<i32>() {
			Ok(n) => total = n,
			_ => total = 0,
		}
	}

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: page < total,
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let html = Request::new(id.as_str(), HttpMethod::Get)
		.html()
		.expect("get manga details html array not an array of nodes");

	let title = html.select(".UAname").text().read();
	let cover = html
		.select(".item__full-sidebar--poster img")
		.attr("abs:src")
		.read();
	let description = html.select(".item__full-description").text().read();

	let mut is_nsfw = false;
	let mut status = "Unknown";

	let mut categories = Vec::new();
	for categ in html.select(".item__full-sidebar--description a").array() {
		let categ_node = categ.as_node().expect("");
		let name = categ_node.text().read();

		if categories.contains(&name) {
			// categories are duplicated on the website
			continue;
		}
		categories.push(name.clone());

		if !is_nsfw {
			// check if only didnt find nsfw category
			is_nsfw = helper::is_nsfw(&name);
		}
		if status == "Unknown" {
			// check if only didnt find status
			status = helper::get_status_string(name.as_str());
		}
	}

	let status_res: MangaStatus = match status {
		"Ongoing" => MangaStatus::Ongoing,
		"Unknown" => MangaStatus::Unknown,
		"Completed" => MangaStatus::Completed,
		"Покинуто" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	};

	let manga = Manga {
		id,
		cover,
		title,
		description,
		categories,
		status: status_res,
		nsfw: if is_nsfw {
			MangaContentRating::Nsfw
		} else {
			MangaContentRating::Safe
		},
		viewer: MangaViewer::Rtl,
		..Default::default()
	};
	Ok(manga)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let html = Request::new(id.as_str(), HttpMethod::Get)
		.html()
		.expect("chapter html array not an array of nodes");

	let linkstocomics = html.select("#linkstocomics");
	let news_id = linkstocomics.attr("data-news_id").read();
	let news_category = linkstocomics.attr("data-news_category").read();
	let this_link = linkstocomics.attr("data-this_link").read();
	let (hash_query, user_hash) = parse_user_hash_and_query(&html)?;

	let body = format!(
		"action=show&news_id={}&news_category={}&this_link={}&{}={}",
		news_id, news_category, this_link, hash_query, user_hash
	);

	let ajax_url = "https://manga.in.ua/engine/ajax/controller.php?mod=load_chapters";
	let response = Request::post(ajax_url)
		.header("Content-Type", "application/x-www-form-urlencoded")
		.body(body.as_bytes())
		.html()?;

	let mut res: Vec<Chapter> = Vec::new();

	let mut chapter_num: f32 = -1.0;

	for chapter in response.select(".ltcitems").array() {
		let node = chapter.as_node().expect("Chapter node expected");
		let a = node.select("a");
		let href = a.attr("href").read();
		let title_raw = a.text().read();

		let mut title = title_raw;
		let date_str = node.select(".ltcright").first().text().0;
		let date = date_str.as_date("dd.MM.yyyy", None, None).unwrap_or(-1.0);
		let scanlator = node
			.select(".ltcright .tooltip .tooltiptext")
			.text()
			.read()
			.replace("Переклад: ", "");

		// Alternative translation don't have chapter number. use previous parsed
		if title.contains("Альтернативний переклад") {
			title = String::from("↑") + &title;
		} else {
			let volume_chapter = title.clone();
			let replaced = volume_chapter.replace("НОВЕ ", "");
			let arr: Vec<_> = replaced.split_whitespace().collect();

			match arr[3].parse::<f32>() {
				Ok(n) => chapter_num = n,
				_ => continue,
			};
		}

		res.push(Chapter {
			id: href.clone(),
			title,
			volume: -1.0,
			chapter: chapter_num,
			url: href,
			date_updated: date,
			scanlator,
			lang: String::from("uk"),
		});
	}

	res.sort_by(|a, b| b.chapter.partial_cmp(&a.chapter).unwrap_or(Ordering::Equal));

	Ok(res)
}

#[get_page_list]
fn get_page_list(_manga_id: String, _chapter_id: String) -> Result<Vec<Page>> {
	let html = Request::new(_chapter_id.as_str(), HttpMethod::Get)
		.html()
		.expect("get page list html array not an array of nodes");

	let base_url = "https://manga.in.ua";
	let endpoint = "engine/ajax/controller.php?mod=load_chapters_image";
	let news_id = html.select("#comics").first().attr("data-news_id").read();
	let (hash_query, user_hash) = parse_user_hash_and_query(&html)?;

	let ajax_url = format!(
		"{}/{}&news_id={}&action=show&{}={}",
		base_url, endpoint, news_id, hash_query, user_hash
	);

	let response = Request::get(ajax_url).header("Referer", base_url).html()?;

	let mut pages: Vec<Page> = Vec::new();
	for (index, result) in response.select("img").array().enumerate() {
		let res_node = result.as_node().expect("");
		let image = res_node.attr("abs:data-src").read();
		pages.push(Page {
			index: index.try_into().unwrap(),
			url: image,
			..Default::default()
		});
	}

	Ok(pages)
}
