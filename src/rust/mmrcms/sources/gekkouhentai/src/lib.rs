#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, Manga, MangaContentRating, MangaPageResult, MangaViewer, Page,
};
use lazy_static::lazy_static;
use mmrcms_template::template::MMRCMSSource;

lazy_static! {
	static ref INSTANCE: MMRCMSSource<'static> = MMRCMSSource {
		base_url: "https://hentai.gekkouscans.com.br",
		lang: "pt-BR",
		category: "Categoria",
		category_parser: |_, categories| {
			let mut viewer = MangaViewer::Rtl;
			for category in categories {
				match category.as_str() {
					"Webtoon" => viewer = MangaViewer::Scroll,
					_ => continue,
				}
			}
			(MangaContentRating::Nsfw, viewer)
		},
		category_mapper: |idx| {
			if idx == 0 {
				String::new()
			} else if (1..=7).contains(&idx) {
				String::from(itoa::Buffer::new().format(idx))
			} else {
				String::from(itoa::Buffer::new().format(idx + 1))
			}
		},
		..Default::default()
	};
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	INSTANCE.get_manga_list(filters, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	INSTANCE.get_manga_details(id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let mut result = INSTANCE.get_chapter_list(id).unwrap_or_default();

	result
		.iter_mut()
		.for_each(|chapter| chapter.title = String::new());
	Ok(result)
}

#[get_page_list]
fn get_page_list(manga_id: String, id: String) -> Result<Vec<Page>> {
	INSTANCE.get_page_list(manga_id, id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	INSTANCE.modify_image_request(request)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	INSTANCE.handle_url(url)
}
