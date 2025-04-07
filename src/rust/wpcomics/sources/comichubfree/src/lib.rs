#![no_std]
pub mod helper;
use crate::helper::*;
use aidoku::{
	error::Result, prelude::*, std::{html::Node, net::Request, String, Vec}, Chapter, DeepLink, Filter,
	FilterType, Listing, Manga, MangaPageResult, Page,
  std::net::HttpMethod
};
use wpcomics_template::{
	helper::{get_tag_id, urlencode},
	template::{self, WPComicsSource, CACHED_MANGA_ID, CACHED_MANGA},
};

const BASE_URL: &str = "https://comichubfree.com";

fn get_instance() -> WPComicsSource {
	WPComicsSource {
		base_url: String::from(BASE_URL),
		listing_mapping: listing_map,

    next_page: "li > a[rel=next]",
		manga_cell: "div.movie-list-index.home-v2 > div.cartoon-box",
		manga_cell_title: "div > h3 > a",
		manga_cell_url: "a.image",
		manga_cell_image: "a.image > img",
		manga_cell_image_attr: "data-src",

		manga_details_title: "h1.movie-title > span",
		manga_details_title_transformer: title_transformer,
		manga_details_cover: "div.movie-image > img",
    manga_details_cover_image_attr: "data-src",
		manga_details_author: "dl.movie-dl > dd:nth-of-type(5)",
		manga_details_author_transformer: |title| title,
		manga_details_description: "#film-content",
		manga_details_tags: "dl.movie-dl > dd:nth-of-type(6)",
		manga_details_tags_splitter: " - ",
		manga_details_status: "dl.movie-dl > dd:nth-of-type(3)",
		manga_details_status_transformer: |title| title,
		manga_details_chapters: "#list > tr",

		chapter_skip_first: false,
		chapter_anchor_selector: "td:nth-of-type(1) > a",
		chapter_date_selector: "td:nth-of-type(2)",
    chapter_title_transformer,
    chapter_raw_title_to_vol_chap: chapter_to_vol_chap,

    paginated_chapter_list: true,
    next_chapter_page: "li > a[rel=next]",

		manga_viewer_page: "div.page-chapter > img",
		manga_viewer_page_url_suffix: "/all",
		page_url_transformer: |url| url,

		..Default::default()
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut title: String = String::new();
	let mut genre: String = String::new();
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				title = urlencode(filter.value.as_string()?.read());
			}
			_ => match filter.name.as_str() {
				"Genre" => {
					genre = get_tag_id(filter.value.as_int().unwrap_or(0));
				}
				_ => continue,
			},
		}
	}
	get_instance().get_manga_list(get_search_url(BASE_URL, &title, &genre,
		page,
	))
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	get_instance().get_manga_listing(listing, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let mut manga = get_instance().get_manga_details(id.clone())?;

  let html = unsafe {
    if CACHED_MANGA_ID.is_some() && CACHED_MANGA_ID.clone().unwrap() == id && CACHED_MANGA.is_some() {
      Node::new(CACHED_MANGA.as_ref().unwrap())?
    } else {
      Request::new(&id, HttpMethod::Get).html()?
    }
  };

  let mut detail_names = Vec::new();
  let mut details = Vec::new();

  let details_node = html.select("dl.movie-dl").first();

  for node in details_node.select("dt").array() {
    let node = node.as_node()?; 
    detail_names.push(node.text().read());
  }

  for node in details_node.select("dd").array() {
    let node = node.as_node()?; 
    details.push(node.text().read());
  }

  for (name, detail) in detail_names.into_iter().zip(details.into_iter()) {
    match name.as_str() {
      "Author:" => manga.author = detail,
      "Status:" => manga.status = (get_instance().status_mapping)(detail),
      "Genres:" => {
        let mut categories = Vec::new();
        detail.split(" - ").into_iter().for_each(|c| categories.push(String::from(c)));
        let (nsfw, viewer) = get_instance().category_parser(&categories);
        manga.categories = categories;
        manga.nsfw = nsfw;
        manga.viewer = viewer;
      },
      &_ => {},
    };
  }

  Ok(manga)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	get_instance().get_chapter_list(id)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	get_instance().get_page_list(chapter_id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	template::modify_image_request(
		String::from(BASE_URL),
		String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.0.4951.54 Safari/537.36 Edg/101.0.1210.39"),
		request,
	)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(url)
}
