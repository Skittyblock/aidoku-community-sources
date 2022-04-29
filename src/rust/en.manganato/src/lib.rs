#![no_std]
use aidoku::{
	prelude::*, error::Result, std::String, std::ObjectRef, std::Vec, std::net::Request, std::net::HttpMethod,
	Filter, FilterType, Listing, Manga, MangaPageResult, Page, MangaStatus, MangaContentRating, MangaViewer, Chapter, DeepLink,
	std::defaults::defaults_get,
};

mod helper;
use helper::*;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    let mut included_tags: Vec<String> = Vec::new();
    let mut excluded_tags: Vec<String> = Vec::new();
    let mut sort: String = String::new();
    let mut title: String = String::new();
    for filter in filters {
		match filter.kind {
			FilterType::Title => {
				title = filter.value.as_string()?.read();
			},
			FilterType::Genre => {
				match filter.value.as_int().unwrap_or(-1) {
					0 => excluded_tags.push(get_tag_id(String::from(&filter.name))),
					1 => included_tags.push(get_tag_id(String::from(&filter.name))),
					_ => continue,
				}
			},
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int().unwrap_or(0);
				let option = match index {
					0 => "",
					1 => "newest",
					2 => "topview",
					3 => "az",
					_ => continue,
				};
				sort = String::from(option)
			},
			_ => continue,
		}
	}

    let mut mangas: Vec<Manga> = Vec::new();
    let url = get_search_url(title, page, included_tags, excluded_tags, sort);
    let html = Request::new(url.as_str(), HttpMethod::Get).html();
    for item in html.select("div.content-genres-item").array() {
        let item_node = item.as_node();
        let title = item_node.select("a").first().attr("title").read();
        let id = string_after(item_node.select("a").first().attr("href").read(), '/', 3);
        let cover = item_node.select("img").first().attr("src").read();
        mangas.push(Manga {
            id,
            cover,
            title,
            author: String::new(),
            artist: String::new(),
            description: String::new(),
            url: String::new(),
            categories: Vec::new(),
            status: MangaStatus::Unknown,
            nsfw: MangaContentRating::Safe,
            viewer: MangaViewer::Default
        });
    }
    let last_page_string = html.select("a.page-last").text().read();
    Ok(MangaPageResult {
        manga: mangas,
        has_more: page < parse_int(String::from(&last_page_string[5..last_page_string.len()-1])),
    })
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
    let mut filters: Vec<Filter> = Vec::new();
	let mut selection = ObjectRef::new();

	selection.set("ascending", false.into());
	selection.set("index", match listing.name.as_str() {
		"Latest Updates" => 0i32.into(),
		"New Manga" => 1i32.into(),
		"Top Manga" => 2i32.into(),
		&_ => 0i32.into()
	});
	filters.push(Filter {
		kind: FilterType::Sort,
		name: String::from("Sort"),
		value: selection.0
	});

	get_manga_list(filters, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
    let url = get_manga_url(id.clone());
    let html = Request::new(url.as_str(), HttpMethod::Get).html();
    let details = html.select("div.panel-story-info");
    let title = details.select("h1").text().read();
    let cover = details.select("span.info-image > img").attr("src").read();
    let author = join_string_array(details.select("td:contains(Author) + td a").array(), String::from(", "));
    let description = details.select("div.panel-story-info-description").text().read();
    let mut categories = Vec::new();
    for node in details.select("td:contains(Genre) + td a").array() {
        categories.push(node.as_node().text().read());
    }
    let status = status_from_string(details.select("td:contains(Status) + td").text().read());
    Ok(Manga {
        id: id.clone(),
        cover,
        title,
        author,
        artist: String::new(),
        description,
        url,
        categories,
        status,
        nsfw: MangaContentRating::Safe,
        viewer: MangaViewer::Default
    })
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
    let mut chapters: Vec<Chapter> = Vec::new();
    let url = get_manga_url(id.clone());
    let html = Request::new(url.clone().as_str(), HttpMethod::Get).html();
    for chapter in html.select("div.panel-story-chapter-list > ul.row-content-chapter > li").array() {
        let chapter_node = chapter.as_node();
        let title = chapter_node.select("a").text().read();
        let chapter_id = string_after(chapter_node.select("a").attr("href").read(), '/', 3);
        let chapter_url = chapter_node.select("a").attr("href").read();
        let chapter_number = get_chapter_number(chapter_id.clone());
        chapters.push(Chapter {
            id: chapter_id,
            title,
            volume: -1.0,
            chapter: chapter_number,
            date_updated: 0.0,
            scanlator: String::new(),
            url: chapter_url,
            lang: String::from("en"),
        });
    }
    Ok(chapters)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
    let mut pages: Vec<Page> = Vec::new();
    let url = get_manga_url(id.clone());
    let html = Request::new(url.as_str(), HttpMethod::Get).html();
    let mut at = 0;
    for page in html.select("div.container-chapter-reader > img").array() {
        let page_node = page.as_node();
        let page_url = page_node.attr("src").read();
        pages.push(Page {
            index: at,
            url: page_url,
            base64: String::new(),
            text: String::new(),
        });
        at += 1;
    }
    Ok(pages)
}

#[modify_image_request] 
fn modify_image_request(request: Request) {
    request.header("Referer", "https://readmanganato.com/");
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let url = &url[26..];
    println!("{}", url);
	if url.starts_with("manga") {
		let end = match url.find("/") {
			Some(end) => end,
			None => url.len(),
		};
		let manga_id = &url[..end];
        println!("{}", manga_id);
		let manga = get_manga_details(String::from(manga_id))?;

		return Ok(DeepLink {
			manga: Some(manga),
			chapter: None,
		});
	}

	Err(aidoku::error::AidokuError { reason: aidoku::error::AidokuErrorKind::Unimplemented })
}