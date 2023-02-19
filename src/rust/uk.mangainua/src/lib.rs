#![no_std]

mod helper;

use aidoku::{
	error::Result,
	prelude::*,
	std::net::Request,
	std::{net::HttpMethod, String, Vec},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut manga_arr: Vec<Manga> = Vec::new();
	let mut total: i32 = 1;

	let baseUrl = String::from("https://manga.in.ua");

	let genres_list = helper::genres_list();

	let mut sortValue: String = String::new();
	let mut searchValue = String::new();
	let mut genreValue = String::new();
	let mut statusValue : String = String::new();

	let mut isCoverDataSrc = false;

	for filter in filters {
		match filter.kind{
			FilterType::Title => {
				searchValue = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				if filter.name.as_str() == "Сортувати" {
					let index = filter.value.as_int()? as usize;

					let option = match index {
						0 => "latest",
						1 => "popular",
						_ => "",
					};
					sortValue = String::from(option);
				}

				if filter.name.as_str() == "Жанри" {
					let index = filter.value.as_int()? as usize;
					match index {
						0 => continue,
						_ => genreValue = String::from(genres_list[index]),
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
					statusValue = String::from(option);
				}
			}
			_ => todo!()
		}
	}

	// in case the user searches on popular - move to latest because popular doesn't have a search
	if sortValue == "popular" && (!searchValue.is_empty() || !genreValue.is_empty() || !statusValue.is_empty()) {
		sortValue = String::from("latest");
	}

	if sortValue == "popular" {
		// ignore page number
		let html = Request::new("https://manga.in.ua/", HttpMethod::Get).html();

		for result in html.select(".owl-carousel .card--big").array() {
			let res_node = result.as_node();

			let href = res_node.select("a").attr("href").read();

			let title = res_node.select(".card__content .card__title a").text().read();
			let cover = res_node.select(".card__cover a figure img").attr("abs:src").read(); 

			let mut isNSFW : MangaContentRating = MangaContentRating::Safe;
			let mut categories: Vec<String> = Vec::new();
			for categRes in res_node.select(".card__category a").array(){
				let categ_node = categRes.as_node();
				let name = categ_node.text().read(); 
				isNSFW = helper::IsNSFW(name.clone());

				categories.push(name);
			}

			manga_arr.push(Manga{
				id: href,
				cover,
				title,
				author: String::new(),
				artist: String::new(),
				description: String::new(),
				url: String::new(),
				categories: categories.clone(),
				status: MangaStatus::Unknown,
				nsfw: isNSFW,
				viewer: MangaViewer::Rtl,
			})
		}
	}
	else {
		let request;

		// have search, genre and status -> ignore status
		if !searchValue.is_empty() && !genreValue.is_empty() {//&& !statusValue.is_empty(){
			let url = format!("{}/mangas/{}", baseUrl, genreValue);

			let body_data = format!("do=search&subaction=search&titleonly=3&story={}", searchValue);

			request = Request::new(url.as_str(), HttpMethod::Post)
						.body(body_data.as_bytes())
						.header("Referer", "https://manga.in.ua");
		}
		// search and status
		else if !searchValue.is_empty() && genreValue.is_empty() && !statusValue.is_empty(){
			let url = format!("{}/{}", baseUrl, statusValue);
			
			let body_data = format!("do=search&subaction=search&titleonly=3&story={}", searchValue);

			request = Request::new(url.as_str(), HttpMethod::Post)
						.body(body_data.as_bytes())
						.header("Referer", "https://manga.in.ua");
		}
		// genre and status -> ignore status
		else if searchValue.is_empty() && !genreValue.is_empty() && !statusValue.is_empty(){
			let url = format!("{}/mangas/{}/page/{}", baseUrl, genreValue, page);
			request = Request::new(url.as_str(), HttpMethod::Get);
			
			isCoverDataSrc = true;
		}
		// only search
		else if !searchValue.is_empty() && genreValue.is_empty() && statusValue.is_empty(){
			let url = format!("{}/index.php?do=search", baseUrl);
			
			let body_data = format!("do=search&subaction=search&story={}&search_start={}", searchValue, page);

			request = Request::new(url.as_str(), HttpMethod::Post)
						.body(body_data.as_bytes())
						.header("Referer", "https://manga.in.ua");
		}
		// only genre
		else if searchValue.is_empty() && !genreValue.is_empty() && statusValue.is_empty(){
			let url = format!("{}/mangas/{}/page/{}", baseUrl, genreValue, page);
			request = Request::new(url.as_str(), HttpMethod::Get);
			
			isCoverDataSrc = true;
		}
		// only status
		else if searchValue.is_empty() && genreValue.is_empty() && !statusValue.is_empty(){
			let url = format!("{}/{}/page/{}", baseUrl, statusValue, page);
			request = Request::new(url.as_str(), HttpMethod::Get);

			isCoverDataSrc = true;
		}
		// should not got here. just in case
		else {
			let url = format!("{}/page/{}", baseUrl, page);
			request = Request::new(url.as_str(), HttpMethod::Get);

			isCoverDataSrc = true;
		}

		let html = request.html();

		for result in html.select(".main .item").array() {
			let res_node = result.as_node();

			let card = res_node.select(".card--big .card__content");

			let title = card.select(".card__title a").text().read();
			
			let href = card.select(".card__title a").attr("href").read();	

			let cover;
			
			if isCoverDataSrc {
				cover = res_node.select(".card--big img").attr("abs:data-src").read();
			}
			else{
				cover = res_node.select(".card--big img").attr("abs:src").read();
			}

			let mut categories: Vec<String> = Vec::new();
			for categRes in res_node.select(".card__category a").array(){
				let categ_node = categRes.as_node();
				let categ_name = categ_node.text().read(); 
				
				categories.push(categ_name);
			}
			
			let mut desc = String::new();
			let mut isNSFW : MangaContentRating = MangaContentRating::Safe;
			for info in card.select(".card__list li").array(){
				let info_node = info.as_node();
				desc = info_node.text().read();
				isNSFW = helper::IsNSFW(desc.clone());
				continue; // need only first element. is there better solution?
			}

			manga_arr.push(Manga{
				id: href,
				cover,
				title,
				author: String::new(),
				artist: String::new(),
				description: desc.clone(),
				url: String::new(),
				categories: categories.clone(),
				status: MangaStatus::Unknown,
				nsfw: isNSFW,
				viewer: MangaViewer::Rtl,
			})
		}
		
		let mut lastPage = String::new();
		for paging_res in html.select(".page-navigation a").array() 
		{
			let paging = paging_res.as_node();
			let st = paging.text().read();

			// first page  -> 	<a>1</1a> ... <a>99</a> <span>Попередня</span> <a>Наступна</a>
			// other pages ->	<a>1</1a> ... <a>99</a> <a>Попередня</a> <a>Наступна</a>
			// on first page "Попередня" (previous) is span, so the latest page number is before "Наступна" (Next) 
			// on other pages "Попередня" and "Наступна" are a elements, so latest page number is before "Попередня"
			if st != "Наступна" && st != "Попередня" {
				lastPage = st;
			}
		}

		match lastPage.parse::<i32>(){
			Ok(n) => total = n,
			Err(_) => todo!(),
		} 
	}
	
	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: page < total,
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let html = Request::new(id.as_str(), HttpMethod::Get).html();

	let title = html.select(".UAname").text().read();
	let cover = html.select(".item__full-sidebar--poster img").attr("abs:src").read();
	let description = html.select(".item__full-description").text().read();

	let mut isNSFW = false;
	let mut status = String::from("Unknown");

	let mut categories = Vec::new();
	for categ in html.select(".item__full-sidebar--description a").array(){
		let categ_node = categ.as_node();
		let name = categ_node.text().read();

		if categories.contains(&name){ // categories are duplicated on the website
			continue;
		}
		categories.push(name.clone());

		if !isNSFW { // check if only didnt find nsfw category
			isNSFW = helper::IsNSFWBool(name.clone());
		}
		if status == "Unknown" { // check if only didnt find status
			status = helper::GetStatusString(name.clone());
		}
	}

	let mut statusRes = MangaStatus::Unknown;
	if status == "Ongoing"{
		statusRes = MangaStatus::Ongoing;
	}
	else if status == "Unknown"{
		statusRes = MangaStatus::Unknown;
	}
	else if status == "Completed"{
		statusRes = MangaStatus::Completed;
	}

	let manga = Manga {
		id,
		cover,
		title,
		author : String::new(),
		artist : String::new(),
		description,
		url : String::new(),
		categories: categories.clone(),
		status: statusRes,
		nsfw: if isNSFW {MangaContentRating::Nsfw} else {MangaContentRating::Safe},
		viewer: MangaViewer::Rtl,
	};
	Ok(manga)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let html = Request::new(id.as_str(), HttpMethod::Get).html();
	
	let mut chapter : f32 = -1.0;
	
	let mut res: Vec<Chapter> = Vec::new();
	for result in html.select(".linkstocomicsblockhidden .ltcitems").array() {
		let res_node = result.as_node();
		let href = res_node.select("a").attr("href").read();
		let mut title = res_node.select("a").text().read();

		let date = res_node.select(".ltcright").first().text()
				.0
				.as_date("dd.MM.yyyy", None, None)
				.unwrap_or(-1.0);

		let scanlator = res_node.select(".ltcright .tooltip .tooltiptext").text().read();

		// Alternative translation don't have chapter number. use previous parsed
		if title.contains("Альтернативний переклад"){
			title = String::from("↑") + &title;
		}
		else
		{	
			// parse volume and chapter.
			let volumeChapter = title.clone();
			let replaced = volumeChapter.replace("НОВЕ ", "");
			let arr: Vec<_> = replaced.split_whitespace().collect();

			match arr[3].parse::<f32>(){
				Ok(n) => chapter = n,
				_e => continue,
			};
		}

		let chapter = Chapter {
			id: href.clone(),
			title,
			volume: -1.0,
			chapter,
			url: href.clone(),
			date_updated: date,
			scanlator,
			lang: String::from("uk"),
		};
		res.push(chapter);
	}
	
	Ok(res)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	let html = Request::new(id.as_str(), HttpMethod::Get).html();

	let mut index : i32 = 0;
	let mut pages: Vec<Page> = Vec::new();
	for result in html.select(".loadcomicsimages img").array() {
		let res_node = result.as_node();
		let image = res_node.attr("abs:data-src").read();
		index += 1;
		pages.push(Page{
			index,
			url: image,
			base64: String::new(),
			text: String::new(),
		});
	}
	Ok(pages)
}
