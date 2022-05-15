use aidoku::{
    error::Result, std::String, std::Vec, std::net::Request, std::net::HttpMethod,
    Listing, Manga, MangaPageResult, Page, MangaStatus, MangaContentRating, MangaViewer, Chapter, DeepLink
};

use crate::helper::{i32_to_string, append_protocol, https_upgrade, extract_i32_from_string};

pub fn get_manga_list(search_url: String, next_page_selector: String, title_transformer: fn(String) -> String) -> Result<MangaPageResult> {
    let mut mangas: Vec<Manga> = Vec::new();
    let mut has_next_page = next_page_selector.len() > 0;
    let html = Request::new(&search_url, HttpMethod::Get).html();
    for item in html.select("div.items > div.row > div.item > figure.clearfix").array() {
        let item_node = item.as_node();
        let title = String::from(item_node.select("figcaption > h3 > a").first().text().read().trim().replacen("http://", "https://", 1));
        let id = https_upgrade(item_node.select("div.image > a").first().attr("href").read());
        let cover = append_protocol(item_node.select("div.image > a > img").first().attr("data-original").read());
        mangas.push(Manga {
            id,
            cover,
            title: title_transformer(title),
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
    if next_page_selector.len() > 0 {
        has_next_page = html.select(&next_page_selector).array().len() > 0;
    }
    Ok(MangaPageResult {
        manga: mangas,
        has_more: has_next_page,
    })
}

pub fn get_manga_listing(base_url: String, listing: Listing, next_page_selector: String, listing_mapping: fn(String) -> String, title_transformer: fn(String) -> String, page: i32) -> Result<MangaPageResult> {
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
    get_manga_list(url, next_page_selector, title_transformer)
}

pub fn get_manga_details(id: String, status_from_string: fn(String) -> MangaStatus, title_transformer: fn(String) -> String) -> Result<Manga> {
    let details = Request::new(id.clone().as_str(), HttpMethod::Get).html();
    let title = details.select("h1.title-detail").text().read();
    let cover = append_protocol(details.select("div.col-image > img").attr("src").read());
    let author = details.select("ul.list-info > li.author > p.col-xs-8").text().read();
    let description = details.select("div.detail-content > p").text().read();
    let mut categories = Vec::new();
    let mut nsfw = MangaContentRating::Safe;
    let mut viewer = MangaViewer::Default;
    for node in details.select("li.kind.row > p.col-xs-8").text().read().split(" - ") {
        let category = String::from(node);
        if category == String::from("Smut") || category == String::from("Mature") || category == String::from("Adult") || category == String::from("18+") {
            nsfw = MangaContentRating::Nsfw;
        } else if category == String::from("Ecchi") || category == String::from("16+") {
            nsfw = MangaContentRating::Suggestive;
        }
        if category.contains("Webtoon") {
            viewer = MangaViewer::Scroll;
        }
        categories.push(category.clone());
    }
    let status = status_from_string(details.select("li.status.row > p.col-xs-8").text().read());
    Ok(Manga {
        id: id.clone(),
        cover,
        title: title_transformer(title),
        author,
        artist: String::new(),
        description,
        url: id.clone(),
        categories,
        status,
        nsfw,
        viewer
    })
}

pub fn get_chapter_list(id: String, title_transformer: fn(String) -> String, skip_first: bool, chapter_date_selector: String, chapter_date_converter: fn(String) -> f64) -> Result<Vec<Chapter>> {
    let mut skipped_first = false;
    let mut chapters: Vec<Chapter> = Vec::new();
    let html = Request::new(id.clone().as_str(), HttpMethod::Get).html();
    let title_untrimmed = title_transformer(html.select("h1.title-detail").text().read());
    let title = title_untrimmed.trim();
    for chapter in html.select("div.list-chapter > nav > ul > li").array() {
        if skip_first && !skipped_first {
            skipped_first = true;
            continue;
        }
        let chapter_node = chapter.as_node();
        let chapter_url = chapter_node.select("div.chapter > a").attr("href").read().replacen("http://", "https://", 1);
        let chapter_id = chapter_url.clone();
        let chapter_title = chapter_node.select("div.chapter > a").text().read();
        let chapter_number = extract_i32_from_string(String::from(title), chapter_title).split(" ").collect::<Vec<&str>>().into_iter().map(|a| a.parse::<f32>().unwrap_or(0.0)).find(|a| *a > 0.0);
        let date_updated = chapter_date_converter(chapter_node.select(&chapter_date_selector).text().read());
        chapters.push(Chapter {
            id: chapter_id,
            title: chapter_title,
            volume: -1.0,
            chapter: chapter_number.unwrap_or(-1.0),
            date_updated,
            scanlator: String::new(),
            url: chapter_url,
            lang: String::from("en"),
        });
    }
    Ok(chapters)
}

pub fn get_page_list(id: String, all_pages_reader_suffix: String, url_transformer: fn(String) -> String) -> Result<Vec<Page>> {
    let mut pages: Vec<Page> = Vec::new();
    let mut url = id.clone();
    url.push_str(&all_pages_reader_suffix);
    let html = Request::new(&url, HttpMethod::Get).html();
    let mut at = 0;
    for page in html.select("div.page-chapter > img").array() {
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
    request.header("Referer", &base_url).header("User-Agent", &user_agent);
}

pub fn handle_url(url: String, status_from_string: fn(String) -> MangaStatus, title_transformer: fn(String) -> String) -> Result<DeepLink> {
    Ok(DeepLink {
        manga: Some(get_manga_details(url.clone(), status_from_string, title_transformer)?),
        chapter: None
    })
}
