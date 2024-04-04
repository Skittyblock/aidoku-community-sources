use aidoku::prelude::{format, println};
use aidoku::std::String;
use aidoku::{Chapter, MangaContentRating, MangaViewer, Page};
use aidoku::{std::ObjectRef, Manga, MangaPageResult};
use aidoku::error::Result;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::helpers::{extract_f32_from_string, id_to_status, siteid_to_domain};
use crate::helpers::SiteId;
extern crate alloc;
                            
static FIRST_SERVER: &str = "https://img2.mixlib.me";
static SECOND_SERVER: &str = "https://img4.mixlib.me";
static COMPRESS_SERVER: &str = "https://img33.imgslib.link";

pub fn parse_manga_list(js: ObjectRef, site: &SiteId) -> Result<MangaPageResult> {
    let has_more = js.get("meta").as_object()?.get("has_next_page").as_bool()?;
    let mangas = js.get("data").as_array()?;
    let mut arr_manga: Vec<Manga> = Vec::new();

    let domain = siteid_to_domain(site);
    for data in mangas {
        if let Ok(data_obj) = data.as_object() {
            let title = match data_obj.get("eng_name").as_string() {
                Ok(x) => x.read(),
                Err(_) => continue
            };
            let id = match data_obj.get("slug_url").as_string() { 
                Ok(x) => x.to_string(),
                Err(_) => continue
            };
            let cover =  match data_obj.get("cover").as_object()?.get("default").as_string() {
                Ok(x) => x.read(),
                Err(_) => continue
            };
            let url = match data_obj.get("slug_url").as_string() {
                Ok(x) => format!("https://{}/ru/manga/{}", domain, x.read()),
                Err(_) => continue
            };
            let status = match data_obj.get("status").as_object()?.get("id").as_int() {
                Ok(x) => id_to_status(x),
                Err(_) => continue
            };

            arr_manga.push( Manga {
                id: id,
                cover: cover,
                title: title,
                author: String::new(),
                artist: String::new(),
                description: String::new(),
                url: url,
                categories: Vec::new(),
                status: status,
                nsfw: MangaContentRating::Safe,
                viewer: aidoku::MangaViewer::Rtl
            })
        }
    }

    Ok(MangaPageResult {
        manga: arr_manga,
        has_more: has_more
    })
}

pub fn parse_manga_details(js: ObjectRef, site: &SiteId) -> Result<Manga> {
    let detail = js.get("data").as_object()?;
    let id = detail.get("slug_url").as_string()?.read();
    println!("{}", id);
    let cover = detail.get("cover").as_object()?.get("default").as_string()?.read();
    println!("{}", cover);
    let title = detail.get("eng_name").as_string()?.read();
    println!("{}", title);
    
    let authors = detail.get("authors").as_array()?;
    let author = authors
		.map(|author| {
			let author_object = author.as_object()?;
			Ok(author_object.get("name").as_string()?.read())
		})
		.map(|a: Result<String>| a.unwrap_or_default())
		.collect::<Vec<String>>()
		.join(", ");
    println!("{}", author);

    let artists = detail.get("artists").as_array()?;
    let artist = artists
		.map(|artist| {
			let artist_object = artist.as_object()?;
			Ok(artist_object.get("name").as_string()?.read())
		})
		.map(|x: Result<String>| x.unwrap_or_default())
		.collect::<Vec<String>>()
		.join(", ");
    println!("{}", artist);

    let description = detail.get("summary").as_string()?.read();
    let url = format!("https://{}/ru/manga/{}", siteid_to_domain(&site), detail.get("slug_url").as_string()?);

    let categories: Vec<String> = detail.get("genres").as_array()?.map(|category| {
        let category_object = category.as_object()?;
        Ok(category_object.get("name").as_string()?.read())
    })
    .map(|x: Result<String>| x.unwrap_or_default())
    .collect::<Vec<String>>();

    let status = id_to_status(detail.get("status").as_object()?.get("id").as_int().unwrap_or_default());
    let nsfw = match site {
        SiteId::MangaLib => MangaContentRating::Safe,
        SiteId::HentaiLib => MangaContentRating::Nsfw,
        SiteId::YaoiLib => MangaContentRating::Nsfw
    };

    let viewer = MangaViewer::Rtl;

    Ok(
        Manga {
            id,
            cover,
            title,
            author,
            artist,
            description,
            url,
            categories,
            status,
            nsfw,
            viewer,
        }
    )
}

pub fn parse_chapter_list(js: ObjectRef, site: &SiteId, id: String) -> Result<Vec<Chapter>> {
    let chapters: Vec<Chapter> = js.get("data").as_array()?.map(|chapter| {
        let chapter_object = chapter.as_object()?;

        // chapter_id: 1#1
        // Scheme: number#volume

        Ok(Chapter {
            id: format!("{}#{}", 
            extract_f32_from_string(String::new(), chapter_object.get("number").as_string()?.read())[0],
            extract_f32_from_string(String::new(), chapter_object.get("volume").as_string()?.read())[0]),

            title:  chapter_object.get("name").as_string().unwrap_or("".into()).to_string(),
            volume: extract_f32_from_string(String::new(), chapter_object.get("volume").as_string()?.read())[0],
            chapter: extract_f32_from_string(String::new(), chapter_object.get("number").as_string()?.read())[0],
            date_updated: -1.0,
            scanlator:  chapter_object.get("branches").as_array()?.get(0).as_object()?.get("user").as_object()?.get("username").as_string()?.read(),
            url: format!("https://{}/{}", siteid_to_domain(site),id),
            lang: "ru".to_string()
        })
    })
    .map(|x: Result<Chapter>| x.unwrap())
    .rev()
    .collect::<Vec<Chapter>>();

    Ok(chapters)
}

pub fn parse_page_list(js: ObjectRef) -> Result<Vec<Page>> {
    let chapters: Vec<Page> = js.get("data").as_object()?.get("pages").as_array()?.map(|page| {
        let page_object = page.as_object()?;
        let url = format!("{}{}", COMPRESS_SERVER, page_object.get("url").as_string()?.read());
        println!("{}", url);
        Ok(Page {
            index: page_object.get("slug").as_int().unwrap() as i32,
            url,
            base64: String::new(),
            text: String::new(),
        })
    })
    .map(|x: Result<Page>| x.unwrap())
    .collect::<Vec<Page>>();

    Ok(chapters)
}