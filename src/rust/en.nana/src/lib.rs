#![no_std]
use aidoku::{
	prelude::*, error::Result, std::String, std::Vec, std::net::Request, std::net::HttpMethod,
	Filter, Manga, MangaPageResult, Page, Chapter
};

mod parser;

const BASE_URL: &str = "https://nana.my.id";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut result: Vec<Manga> = Vec::new();
	let mut url = String::new();

	parser::get_filtered_url(filters, page, &mut url);
	
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	let html_next_page = Request::new(url.as_str(), HttpMethod::Get).html()?;

	parser::parse_search(html, &mut result);

	Ok(MangaPageResult {
		manga: result,
		has_more: html_next_page.select("a.paginate_button.current + a.paginate_button").array().len() > 0,
	})
}

#[no_mangle]
pub unsafe extern "C" fn get_manga_details(manga_rid: i32) -> i32 {
    let manga = aidoku::std::ValueRef::new(manga_rid).as_object().unwrap();
	let url = format!("{}/reader/{}", BASE_URL, manga.get("id").as_string().unwrap().read());
	let html = Request::new(url.as_str(), HttpMethod::Get).html().expect("Node");

    let mut categories: Vec<String> = Vec::new();
    match manga.get("tags").as_array() {
        Ok(tags) => {
            for tag in tags {
                categories.push(tag.as_string().expect("String").read());
            }
        },
        Err(_) => {}
    }

	let img = html.select("a#display img").attr("src").read();
	let img_url = if img.starts_with("/") {
		format!("{}{}", BASE_URL, img)
	} else {
		img
	}.replace("/image/page", "/image/thumbnails");

	let cover = match manga.get("cover").as_string() {
        Ok(cover) => cover.read(),
        Err(_) => String::from(img_url) // get new cover
    };

	let title = match manga.get("title").as_string() {
        Ok(title) => title.read(),
        Err(_) => String::from(html.select("#archivePagesOverlay .spanh3reader").text().read().as_str().trim()) // get new title
    };

    let author = match manga.get("author").as_string() {
        Ok(author) => author.read(),
        Err(_) => String::new()
    };

    let url = match manga.get("url").as_string() {
        Ok(url) => url.read(),
        Err(_) => url
    };

    Manga {
        id: manga.get("id").as_string().unwrap().read(),
        cover,
        title,
        author,
        artist: String::new(),
        description: String::new(),
        url,
        categories,
        status: aidoku::MangaStatus::Completed,
        nsfw: aidoku::MangaContentRating::Nsfw,
        viewer: aidoku::MangaViewer::Scroll
    }.create()
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/reader/{}", BASE_URL, id);

	Ok(Vec::from([Chapter {
		id,
		title: String::from("Chapter 1"),
		volume: -1.0,
		chapter: 1.0,
		url,
		date_updated: 0.0,
		scanlator: String::new(),
		lang: String::from("en"),
	}]))
}

#[get_page_list]
fn get_page_list(chapter_id: String, _manga_id: String) -> Result<Vec<Page>> {
	let url = format!("{}/api/archives/{}/extractthumbnails", BASE_URL, &chapter_id);

	let request = Request::new(&url, HttpMethod::Get).header("User-Agent", "Aidoku");
	let json = request.json()?.as_object()?;
	
	return parser::get_page_list(json);
}

pub fn modify_image_request(request: Request) {
	request.header("Referer", BASE_URL);
}