#![no_std]

mod helper;

use aidoku::{
    error::Result,
    prelude::*,
    std::net::Request,
    std::String,
    std::Vec,
    std::{net::HttpMethod, ObjectRef},
    Chapter, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus,
    MangaViewer, Page,
};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    let mut manga_arr = Vec::new();
    let mut total: i32 = 1;

    let mut query = String::new();
    let mut sort = String::new();

    let mut included_tags: Vec<String> = Vec::new();
    let mut excluded_tags: Vec<String> = Vec::new();

    for filter in filters {
        match filter.kind {
            FilterType::Title => {
                query = helper::urlencode(filter.value.as_string()?.read());
            },
            FilterType::Genre => {
                if let Ok(tag_id) = filter.object.get("id").as_string() {
                    match filter.value.as_int().unwrap_or(-1) {
                        0 => excluded_tags.push(tag_id.read()),
                        1 => included_tags.push(tag_id.read()),
                        _ => continue,
                    }
                }
            },
            FilterType::Sort => {
                let value = match filter.value.as_object() {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                let index = value.get("index").as_int().unwrap_or(0);
                let option = match index {
                    0 => "id",
                    1 => "title",
                    2 => "created_at",
                    3 => "published_at",
                    4 => "pages",
                    _ => "",
                };
                sort = String::from(option)
            },
            _ => continue,
        }
    }

    let mut url = format!(
        "https://koushoku.org/search?sort={}&page={}&q={}",
        sort,
        helper::i32_to_string(page),
        query
    );
    let mut query_params = String::new();
    if !included_tags.is_empty() {
        query_params.push_str("tag&:");
        query_params.push_str(&included_tags.join(","));
    }
    if !excluded_tags.is_empty() {
        query_params.push_str("-tag:");
        query_params.push_str(&excluded_tags.join(","));
    }
    url.push_str(helper::urlencode(query_params).as_str());

    let html = Request::new(url.as_str(), HttpMethod::Get).html();

    for result in html.select(".entries .entry a").array() {
        let result_node = result.as_node();
        let manga_url = result_node.attr("href").read();
        if manga_url.is_empty() {
            continue;
        }

        let title = result_node.select(".metadata h3.title span").text().read();
        let cover = result_node
            .select("figure.thumbnail img")
            .attr("src")
            .read();

        // after second slash in url, get integer
        let manga_id = manga_url
            .split("/")
            .nth(2)
            .unwrap_or("")
            .parse::<i32>()
            .unwrap_or(0);
        manga_arr.push(Manga {
            id: helper::i32_to_string(manga_id),
            cover,
            title,
            author: String::new(),
            artist: String::new(),
            description: String::new(),
            url: String::new(),
            categories: Vec::new(),
            status: MangaStatus::Completed,
            nsfw: MangaContentRating::Nsfw,
            viewer: MangaViewer::Rtl,
        });

        for pagination in html.select("nav.pagination ul li.last a").array() {
            let paging_node = pagination.as_node();
            let last_page_link = paging_node.attr("href").read();
            let last_page = helper::get_page(last_page_link);

            if last_page > total {
                total = last_page
            }
        }
    }

    Ok(MangaPageResult {
        manga: manga_arr,
        has_more: page < total,
    })
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
    let mut filters: Vec<Filter> = Vec::new();

    let mut or = ObjectRef::new();

    or.set("ascending", false.into());

    or.set(
        "index",
        3i32.into(),
    );

    filters.push(Filter {
        name: String::from("sort"),
        kind: FilterType::Sort,
        value: or.0,
        object: ObjectRef::new(),
    });

    // call get manga list after doing some filters here
    get_manga_list(filters, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
    let url = format!("https://koushoku.org/archive/{}/index.json", &id);

    let json = Request::new(url.as_str(), HttpMethod::Get)
        .json()
        .as_object()?;

    let cover = helper::get_cover_url(id.clone());
    let title = json.get("title").as_string()?.read();
    let artists = json.get("artists").as_array()?;
    let artist = artists.get(0).as_object()?;
    let author = artist.get("name").as_string()?.read();
    let tag_list = json.get("tags").as_array()?;
    let mut categories: Vec<String> = Vec::new();
    for tag in tag_list {
        let tag = tag.as_object()?;
        let name = tag.get("name").as_string()?.read();
        categories.push(name);
    }

    let share_url = format!("https://koushoku.org/archive/{}", &id);

    let manga = Manga {
        id,
        cover,
        title,
        author,
        artist: String::new(),
        description: String::new(),
        url: share_url,
        categories,
        status: MangaStatus::Completed,
        nsfw: MangaContentRating::Nsfw,
        viewer: MangaViewer::Rtl,
    };

    Ok(manga)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
    let url = format!("https://koushoku.org/archive/{}/index.json", id.clone());

    let json = Request::new(url.as_str(), HttpMethod::Get)
        .json()
        .as_object()?;

    let date_updated = json.get("updatedAt").as_float()?;

    let mut chapters: Vec<Chapter> = Vec::new();
    chapters.push(Chapter {
        id,
        title: String::new(),
        volume: -1.0,
        chapter: 1.0,
        url,
        date_updated,
        scanlator: String::new(),
        lang: String::from("en"),
    });

    Ok(chapters)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
    let url = format!("https://koushoku.org/archive/{}/index.json", id.clone());

    let mut pages: Vec<Page> = Vec::new();

    let json = Request::new(url.as_str(), HttpMethod::Get)
        .json()
        .as_object()?;

    let pages_total = json.get("pages").as_int()?;

    for i in 1..=pages_total {
        let img_url = format!("https://cdn.koushoku.org/data/{}/{}/512.png", id, i);

        pages.push(Page {
            index: i as i32,
            url: img_url,
            base64: String::new(),
            text: String::new(),
        });
    }

    Ok(pages)
}
