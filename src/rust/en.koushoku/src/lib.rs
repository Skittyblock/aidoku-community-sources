#![no_std]

mod helper;

use aidoku::{
    error::Result,
    prelude::*,
    std::net::Request,
    std::String,
    std::Vec,
    std::{net::HttpMethod},
    Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
    MangaViewer, Page, DeepLink,
};

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    let mut manga_arr = Vec::new();
    let mut total: i32 = 1;

    let mut query = String::new();
    let mut sort = String::new();
    let mut ascending = false;

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
                ascending = value.get("ascending").as_bool().unwrap_or(false);
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

    let url = helper::build_search_url(query, sort, included_tags, excluded_tags, ascending, page.clone());
   

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

        let manga_id = helper::get_manga_id_from_path(manga_url);
     
        
        manga_arr.push(Manga {
            id: manga_id,
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
    }
    // check if paging node exists
        
        let paging = html.select("nav.pagination ul li.last a");
        if !paging.html().read().is_empty() {
            let last_page = helper::get_page(paging.last().attr("href").read());
            
            if last_page > total {
                total = last_page
            }
        }



    Ok(MangaPageResult {
        manga: manga_arr,
        has_more: page < total,
    })
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

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
    let id = helper::get_manga_id(url);
    let manga = get_manga_details(id.clone())?;
    return Ok(DeepLink {
        manga: Some(manga),
        chapter: None,
    });
}