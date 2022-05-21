#![no_std]
mod helper;
use crate::helper::{extract_f32_from_string, genre_map, status_from_string, urlencode};
use aidoku::{
    error::Result,
    prelude::*,
    std::{
        json::parse,
        net::{HttpMethod, Request},
        String, Vec,
    },
    Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
    MangaViewer, Page,
};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    let mut included_tags: Vec<String> = Vec::new();
    let mut excluded_tags: Vec<String> = Vec::new();
    let mut title: String = String::new();
    let mut author: String = String::new();
    let mut status: i32 = -1;
    for filter in filters {
        match filter.kind {
            FilterType::Title => {
                title = urlencode(filter.value.as_string()?.read());
            }
            FilterType::Author => {
                author = urlencode(filter.value.as_string()?.read());
            }
            FilterType::Genre => {
                let genre_id = genre_map(filter.name);
                if genre_id.as_str() != "" {
                    match filter.value.as_int().unwrap_or(-1) {
                        0 => excluded_tags.push(genre_id),
                        1 => included_tags.push(genre_id),
                        _ => continue,
                    }
                } else {
                    continue;
                }
            }
            _ => match filter.name.as_str() {
                "Trạng thái" => {
                    status = filter.value.as_int().unwrap_or(-1) as i32;
                }
                _ => continue,
            },
        }
    }

	let url = if included_tags.len() > 0
        || excluded_tags.len() > 0
        || title.len() > 0
        || author.len() > 0
    {
        let included_tags_string = if included_tags.len() > 0 {
            included_tags.join(",")
        } else {
            String::from("-1")
        };
        let excluded_tags_string = if excluded_tags.len() > 0 {
            excluded_tags.join(",")
        } else {
            String::from("-1")
        };
        format!(
			// This page has a scanlator search feature, maybe add that when Aidoku has it
			"https://blogtruyen.vn/timkiem/nangcao/1/{status}/{}/{}?txt={title}&aut={author}&p={page}&gr=",
			included_tags_string,
			excluded_tags_string,
		)
	} else {
		format!("https://blogtruyen.vn/ajax/Category/AjaxLoadMangaByCategory?id=0&orderBy=5&p={page}")
	};
	let html = Request::new(url.as_str(), HttpMethod::Get).html();
	let mut manga_arr: Vec<Manga> = Vec::new();
	for (url, info) in html.select("div.list > p > span.tiptip > a").array().zip(
		html.select("div.list > div.tiptip-content > div.row")
			.array(),
	) {
		let url_node = url.as_node();
		let info_node = info.as_node();
		let title = info_node.select("div.col-sm-8 > div.al-c").text().read();
		let description = info_node.select("div.col-sm-8 > div.al-j").text().read();
		let cover = info_node.select("div.col-sm-4 > img").attr("src").read();
		let id = url_node.attr("href").read();
		let url = format!("https://blogtruyen.vn{id}");
		manga_arr.push(Manga {
			id,
			cover: String::from(cover),
			title: String::from(title.trim()),
			author: String::new(),
			artist: String::new(),
			description: String::from(description.trim()),
			url,
			categories: Vec::new(),
			status: MangaStatus::Unknown,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Rtl,
		});
	}
	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: html.select("a[title=\"Trang cuối\"]").array().len() > 0 || html.select("a:contains([cuối])").array().len() > 0,
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
    let url = format!("https://blogtruyen.vn{id}");
    let html = Request::new(url.as_str(), HttpMethod::Get).html();
    let title = html
        .select("div.thumbnail > img")
        .attr("alt")
        .read()
        .replace("truyện tranh", "");
    let cover = html.select("div.thumbnail > img").attr("src").read();
    let description = html
        .select("section.manga-detail > div.detail > div.content")
        .text()
        .read();
    let author = html
        .select("div.description > p:contains(Tác giả) > a")
        .array()
        .map(|val| val.as_node().text().read())
        .collect::<Vec<String>>()
        .join(", ");
    let categories = html
        .select("span.category > a")
        .array()
        .map(|val| val.as_node().text().read())
        .collect::<Vec<String>>();
    let status = status_from_string(
        html.select("p:contains(Trạng thái) > span.color-red")
            .text()
            .read(),
    );
    let mut viewer = MangaViewer::Rtl;
    let mut nsfw = MangaContentRating::Safe;
    for category in &categories {
        match category.as_str() {
            "Smut" | "Mature" | "18+" => nsfw = MangaContentRating::Nsfw,
            "Ecchi" | "16+" => {
                nsfw = match nsfw {
                    MangaContentRating::Nsfw => MangaContentRating::Nsfw,
                    _ => MangaContentRating::Suggestive,
                }
            }
            "Webtoon" | "Manhwa" | "Manhua" => viewer = MangaViewer::Scroll,
            _ => continue,
        }
    }
    Ok(Manga {
        id,
        cover,
        title: String::from(title.trim()),
        author,
        artist: String::new(),
        description,
        url,
        categories,
        status,
        nsfw,
        viewer,
    })
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
    let url = format!("https://blogtruyen.vn{id}");
    let html = Request::new(url.as_str(), HttpMethod::Get).html();
    let mut scanlator = html
        .select("span.translater")
        .array()
        .map(|val| val.as_node().text().read())
        .collect::<Vec<String>>();
	scanlator.dedup();
	let scanlator_string = scanlator.join(", ");
    let manga_title = html
        .select("div.thumbnail > img")
        .attr("alt")
        .read()
        .replace("truyện tranh", "");
    let mut chapter_arr: Vec<Chapter> = Vec::new();
    for chapter_item in html.select("p[id^=\"chapter\"]").array() {
        let chapter_node = chapter_item.as_node();
        let chapter_id = chapter_node.select("span.title > a").attr("href").read();
        let mut title = chapter_node
            .select("span.title > a")
            .text()
            .read()
            .replace(manga_title.trim(), "");
        let chapter = extract_f32_from_string(String::from(""), String::from(&title));
		let splitter = format!(" {}", chapter);
		if title.contains(&splitter) {
			let split = title.splitn(2, &splitter).collect::<Vec<&str>>();
			title = String::from(split[1]).replacen(|char| {
				return char == ':' || char == '-'
			}, "", 1);
		}
        let date_updated = chapter_node
            .select("span.publishedDate")
            .text()
            .0
            .as_date("dd/MM/yyyy HH:mm", Some("en_US"), Some("Asia/Ho_Chi_Minh"))
            .unwrap_or(-1.0);
        let url = format!("https://blogtruyen.vn{chapter_id}");
        chapter_arr.push(Chapter {
            id: chapter_id,
            title: String::from(title.trim()),
            volume: -1.0,
            chapter,
            date_updated,
            scanlator: String::from(&scanlator_string),
            url,
            lang: String::from("vi"),
        });
    }
    Ok(chapter_arr)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
    let url = format!("https://blogtruyen.vn{id}");
    let html = Request::new(url.as_str(), HttpMethod::Get).html();
    let mut page_arr: Vec<Page> = Vec::new();
    let mut page_index = 0;
    for page_item in html.select("article#content > img").array() {
        let page_node = page_item.as_node();
        page_arr.push(Page {
            index: page_index,
            url: page_node.attr("src").read(),
            base64: String::new(),
            text: String::new(),
        });
        page_index += 1;
    }

    // some chapters push pages from script
    let script = html.select("article#content > script").text().read();
    if script != "" && script.contains("listImageCaption") {
        if let Some(images_array_string) = script.split(";").collect::<Vec<&str>>()[0]
            .split("=")
            .collect::<Vec<&str>>()
            .last()
        {
            let val = parse(images_array_string.as_bytes());
            if let Ok(images_array) = val.as_array() {
                images_array.for_each(|val| {
                    if let Ok(url) = val.as_string() {
                        page_arr.push(Page {
                            index: page_index,
                            url: url.read(),
                            base64: String::new(),
                            text: String::new(),
                        });
                        page_index += 1;
                    }
                })
            }
        }
    }
    Ok(page_arr)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
    request
		.header("Referer", "https://blogtruyen.vn")
		.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.0.4951.64 Safari/537.36 Edg/101.0.1210.47");
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
    // https://blogtruyen.vn/19588/uchi-no-hentai-maid-ni-osowareteru
	// 'https:', '', 'blogtruyen.vn', '19588', 'uchi-no-hentai-maid-ni-osowareteru'
    let split = url.split("/").collect::<Vec<&str>>();
	let id = format!("/{}", &split[3..].join("/"));
    if id.contains("chuong") {
		let html = Request::new(url.as_str(), HttpMethod::Get).html();
		let manga_id = html.select("div.breadcrumbs > a:nth-child(2)").attr("href").read();
		let manga = get_manga_details(manga_id)?;
		Ok(DeepLink {
			manga: Some(manga),
			chapter: None,
		})
	} else {
		let manga = get_manga_details(id)?;
		Ok(DeepLink {
			manga: Some(manga),
			chapter: None,
		})
	}
}
