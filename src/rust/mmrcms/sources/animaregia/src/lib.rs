#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{net::Request, String, Vec},
	Chapter, DeepLink, Filter, Manga, MangaPageResult, Page,
};
use lazy_static::lazy_static;
use mmrcms_template::template::MMRCMSSource;

lazy_static! {
	static ref INSTANCE: MMRCMSSource = MMRCMSSource {
		base_url: "http://animaregia.net",
		lang: "pt-BR",
		category: "Categoria",
		details_title_selector: "h1.widget-title",
		detail_description: "Sumário",
		detail_status_ongoing: "Ativo",
		detail_status_complete: "Concluído",
		..Default::default()
	};
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let result = INSTANCE
		.get_manga_list(filters, page)
		.unwrap_or(MangaPageResult {
			manga: Vec::new(),
			has_more: false,
		});

	Ok(MangaPageResult {
		manga: result
			.manga
			.into_iter()
			.map(|manga| Manga {
				id: manga.id,
				cover: manga.cover,
				title: manga.title.replace(" (pt-br)", ""),
				author: manga.author,
				artist: manga.artist,
				description: manga.description,
				url: manga.url,
				categories: manga.categories,
				status: manga.status,
				nsfw: manga.nsfw,
				viewer: manga.viewer,
			})
			.collect::<Vec<_>>(),
		has_more: result.has_more,
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	match INSTANCE.get_manga_details(id) {
		Ok(mut result) => {
			result.title = result.title.replace(" (pt-br)", "");
			Ok(result)
		}
		Err(error) => Err(error),
	}
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	INSTANCE.get_chapter_list(id)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	INSTANCE.get_page_list(id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	INSTANCE.modify_image_request(request)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	INSTANCE.handle_url(url)
}
