#![no_std]
extern crate alloc;

use alloc::string::ToString;
use aidoku::{
    error::Result,
    prelude::*,
    std::net::{Request, HttpMethod},
    std::String,
    std::Vec,
    std::ObjectRef,
    Chapter,
    Filter,
    FilterType,
    Manga,
    MangaPageResult,
    Page,
    MangaStatus,
    MangaContentRating,
    MangaViewer,
};

const API_URL: &str = "https://api.shngm.io";
const CDN_URL: &str = "https://storage.shngm.id";
const BASE_URL: &str = "https://app.shinigami.asia";

// Todo: Search, Tags, Filters

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    let mut url = format!(
        "{}/v1/manga/list?type=project&page={}&page_size=30&is_update=true&sort=latest&sort_order=desc",
        API_URL, page
    );
    
    // Handle search and filters here
    for filter in filters {
        match filter.kind {
            FilterType::Title => {
                if let Ok(query) = filter.value.as_string() {
                    url.push_str(&format!("&q={}", query.read()));
                }
            },
            _ => continue,
        }
    }

    let json = Request::new(url, HttpMethod::Get)
        .header("Accept", "application/json")
        .header("Origin", BASE_URL)
        .header("DNT", "1")
        .header("Sec-GPC", "1")
        .json()?;

    let data = json.as_object()?;
    
    if data.get("retcode").as_int()? != 0 {
        return Ok(MangaPageResult { manga: Vec::new(), has_more: false });
    }

    let manga_array = data.get("data").as_array()?;
    let mut mangas = Vec::new();

    for manga in manga_array {
        let manga_obj = manga.as_object()?;
        let title = manga_obj.get("title").as_string()?.read();
        let _alt_title = manga_obj.get("alternative_title").as_string()?.read();
        
        let cover = if let Ok(url) = manga_obj.get("cover_image_url").as_string() {
            url.read()
        } else {
            manga_obj.get("cover_image_url").as_string()?.read()
        };

        let taxonomy = manga_obj.get("taxonomy").as_object()?;
        let mut genres = Vec::new();
        if let Ok(genre_array) = taxonomy.get("Genre").as_array() {
            for genre in genre_array {
                if let Ok(genre_obj) = genre.as_object() {
                    if let Ok(genre_name) = genre_obj.get("name").as_string() {
                        genres.push(genre_name.read());
                    }
                }
            }
        }

        mangas.push(Manga {
            id: manga_obj.get("manga_id").as_string()?.read(),
            title,
            author: get_taxonomy_names(&taxonomy, "Author"),
            artist: get_taxonomy_names(&taxonomy, "Artist"),
            description: manga_obj.get("description").as_string()?.read(),
            url: manga_obj.get("manga_id").as_string()?.read(),
            cover,
            categories: genres,
            status: match manga_obj.get("status").as_int()? {
                1 => MangaStatus::Ongoing,
                2 => MangaStatus::Completed,
                _ => MangaStatus::Unknown,
            },
            nsfw: MangaContentRating::Safe,
            viewer: MangaViewer::Rtl,
            ..Default::default()
        });
    }

    let meta = data.get("meta").as_object()?;
    let current_page = meta.get("page").as_int()?;
    let total_pages = meta.get("total_page").as_int()?;
    
    Ok(MangaPageResult {
        manga: mangas,
        has_more: current_page < total_pages,
    })
}

fn get_taxonomy_names(taxonomy: &ObjectRef, key: &str) -> String {
    if let Ok(array) = taxonomy.get(key).as_array() {
        let mut names = Vec::new();
        for item in array {
            if let Ok(obj) = item.as_object() {
                if let Ok(name) = obj.get("name").as_string() {
                    names.push(name.read());
                }
            }
        }
        names.join(", ")
    } else {
        String::new()
    }
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
    let url = format!(
        "{}/v1/chapter/{}/list?page=1&page_size=3000&sort_by=chapter_number&sort_order=desc",
        API_URL, id
    );
    let json = Request::new(url, HttpMethod::Get)
        .header("Accept", "application/json")
        .header("Origin", BASE_URL)
        .json()?;

    let data = json.as_object()?;
    if data.get("retcode").as_int()? != 0 {
        return Ok(Vec::new());
    }

    let chapter_list = data.get("data").as_array()?;
    let mut chapters = Vec::new();

    for chapter in chapter_list {
        let chapter_obj = chapter.as_object()?;
        let chapter_number = chapter_obj.get("chapter_number").as_float()? as f32;
        chapters.push(Chapter {
            id: chapter_obj.get("chapter_id").as_string()?.read(),
            title: format!(
                "Chapter {}", 
                chapter_number.to_string().replace(".0", "")
            ),
            chapter: chapter_number,
            url: format!("{}/chapter/{}", BASE_URL, chapter_obj.get("chapter_id").as_string()?.read()),
            date_updated: parse_date(chapter_obj.get("release_date").as_string()?.read()),
            ..Default::default()
        });
    }

    Ok(chapters)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
    let url = format!("{}/v1/manga/detail/{}", API_URL, id);
    let json = Request::new(url, HttpMethod::Get)
        .header("Accept", "application/json")
        .header("Origin", BASE_URL)
        .json()?;

    let data = json.as_object()?;
    if data.get("retcode").as_int()? != 0 {
        return Err(aidoku::error::AidokuError { 
            reason: aidoku::error::AidokuErrorKind::NodeError(aidoku::error::NodeError::ParseError) 
        });
    }

    let manga_obj = data.get("data").as_object()?;
    let title = manga_obj.get("title").as_string()?.read();
    let _alt_title = manga_obj.get("alternative_title").as_string()?.read();
    
    let cover = if let Ok(url) = manga_obj.get("cover_portrait_url").as_string() {
        if !url.read().is_empty() {
            url.read()
        } else {
            manga_obj.get("cover_image_url").as_string()?.read()
        }
    } else {
        manga_obj.get("cover_image_url").as_string()?.read()
    };

    let taxonomy = manga_obj.get("taxonomy").as_object()?;
    let mut genres = Vec::new();
    if let Ok(genre_array) = taxonomy.get("Genre").as_array() {
        for genre in genre_array {
            if let Ok(genre_obj) = genre.as_object() {
                if let Ok(genre_name) = genre_obj.get("name").as_string() {
                    genres.push(genre_name.read());
                }
            }
        }
    }

    Ok(Manga {
        id: manga_obj.get("manga_id").as_string()?.read(),
        title,
        author: get_taxonomy_names(&taxonomy, "Author"),
        artist: get_taxonomy_names(&taxonomy, "Artist"),
        description: manga_obj.get("description").as_string()?.read(),
        url: manga_obj.get("manga_id").as_string()?.read(),
        cover,
        categories: genres,
        status: match manga_obj.get("status").as_int()? {
            1 => MangaStatus::Ongoing,
            2 => MangaStatus::Completed,
            _ => MangaStatus::Unknown,
        },
        nsfw: MangaContentRating::Safe,
        viewer: MangaViewer::Rtl,
        ..Default::default()
    })
}

#[modify_image_request]
fn modify_image_request(request: Request) {
    request
        .header("Accept", "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8")
        .header("DNT", "1")
        .header("Referer", BASE_URL)  // Changed from format!() to direct string
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-GPC", "1");
}

// Helper function to parse date
fn parse_date(date_str: String) -> f64 {
    if date_str.is_empty() {
        return 0.0;
    }

    let parts: Vec<&str> = date_str.split(['T', 'Z', '-', ':'].as_ref()).collect();
    if parts.len() >= 6 {
        if let (Ok(year), Ok(month), Ok(day), Ok(hour), Ok(min), Ok(sec)) = (
            parts[0].parse::<i64>(),
            parts[1].parse::<i64>(),
            parts[2].parse::<i64>(),
            parts[3].parse::<i64>(),
            parts[4].parse::<i64>(),
            parts[5].parse::<i64>(),
        ) {
            // Convert to Unix timestamp
            let days_since_epoch = (year - 1970) * 365 + month * 30 + day;
            let seconds = days_since_epoch * 24 * 60 * 60 + hour * 60 * 60 + min * 60 + sec;
            return seconds as f64;
        }
    }
    0.0
}

#[get_page_list]
fn get_page_list(_manga_id: String, id: String) -> Result<Vec<Page>> {
    let url = format!("{}/v1/chapter/detail/{}", API_URL, id);
    let json = Request::new(url, HttpMethod::Get)
        .header("Accept", "application/json")
        .header("Origin", BASE_URL)
        .json()?;

    let data = json.as_object()?;
    if data.get("retcode").as_int()? != 0 {
        return Ok(Vec::new());
    }

    let chapter_data = data.get("data").as_object()?;
    let chapter = chapter_data.get("chapter").as_object()?;
    let path = chapter.get("path").as_string()?.read();
    let pages = chapter.get("data").as_array()?;

    let mut result = Vec::new();
    for (index, page) in pages.enumerate() {
        result.push(Page {
            index: index as i32,
            url: format!("{}{}{}", CDN_URL, path, page.as_string()?.read()),
            ..Default::default()
        });
    }

    Ok(result)
}