#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{html::Node, net::Request, String, Vec},
	Chapter, DeepLink, Filter, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};
use lazy_static::lazy_static;
use mmrcms_template::{
	helper::text_with_newlines,
	template::{cache_manga_page, MMRCMSSource, CACHED_MANGA},
};

lazy_static! {
	static ref INSTANCE: MMRCMSSource = MMRCMSSource {
		base_url: "https://onma.me",
		lang: "ar",

		category: "الفئة",
		tags: "العلامات",
		category_parser: |_, categories| {
			let mut nsfw = MangaContentRating::Safe;
			let mut viewer = MangaViewer::Rtl;
			for category in categories {
				match category.as_str() {
					// "Sexual perversion" | "Mature"
					"انحراف جنسي" | "ناضج" => nsfw = MangaContentRating::Nsfw,
					// Webtoon
					"ويب تون" => viewer = MangaViewer::Scroll,
					_ => continue,
				}
			}
			(nsfw, viewer)
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
	let url = format!("{}/{}/{}", INSTANCE.base_url, INSTANCE.manga_path, id);
	cache_manga_page(&url);
	let html = Node::new_with_uri(unsafe { &CACHED_MANGA.clone().unwrap() }, &url);

	let title = html.select("div.panel-heading").text().read();
	let cover = html.select("img.img-thumbnail").attr("abs:src").read();
	let description = text_with_newlines(html.select("div.well > p"));
	let mut manga = Manga {
		id,
		title,
		cover,
		description,
		url,
		..Default::default()
	};
	for elem in html.select("div.col-md-6 h3").array() {
		let node = elem.as_node();
		let text = node.text().read().to_lowercase();
		let end = text.find(" : ").unwrap_or(0);
		match &text.as_str()[..end] {
			"النوع" => manga.categories.push(node.select("div").text().read()),
			"المؤلف" => manga.author = node.select("div").text().read(),
			"الرسام" => manga.artist = node.select("div").text().read(),
			"الحالة" => {
				manga.status = match node.select("span.label").text().read().trim() {
					"مستمرة" => MangaStatus::Ongoing,
					"مكتملة" => MangaStatus::Completed,
					_ => MangaStatus::Unknown,
				}
			}
			"التصنيفات" => node
				.select("a")
				.array()
				.for_each(|elem| manga.categories.push(elem.as_node().text().read())),
			_ => continue,
		}
	}
	(manga.nsfw, manga.viewer) = (INSTANCE.category_parser)(&html, manga.categories.clone());
	if html.select("div.alert.alert-danger").array().len() > 0 {
		manga.nsfw = MangaContentRating::Nsfw;
	}
	Ok(manga)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let mut result = INSTANCE.get_chapter_list(id).unwrap_or_default();

	result.iter_mut().for_each(|chapter| {
		chapter.title = String::from(
			chapter.title.split(" : ").collect::<Vec<_>>()[1]
				.replace("تحميل", "")
				.trim(),
		)
	});
	Ok(result)
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
