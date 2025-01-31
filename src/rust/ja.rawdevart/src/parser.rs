use crate::alloc::string::ToString;
use crate::{BASE_URL, PAGE_URL};
use aidoku::{
	error::Result,
	helpers::substring::Substring,
	prelude::*,
	std::{html::Node, ObjectRef, String, Vec},
	Chapter, Manga, MangaContentRating, MangaStatus, MangaViewer, Page,
};

// parse manga id from cover url
// e.g. from https://rawdevart.art/img/thumb/42/cb/8872/yasashii-kazoku-to-takusan-no-mofumofu-ni-kakomarete-8872-200x297.webp
//        to yasashii-kazoku-to-takusan-no-mofumofu-ni-kakomarete-
fn parse_manga_url_id<'a>(cover_url: &'a str, id: &'a str) -> Option<&'a str> {
	cover_url
		.substring_after(&format!("{}/", id))
		.and_then(|str| str.substring_before(id))
}

// Parse manga with title and cover from basic manga object
pub fn parse_basic_manga(manga_object: &ObjectRef) -> Result<Manga> {
	let id = manga_object.get("manga_id").as_int()?.to_string();
	let cover = manga_object.get("manga_cover_img_full").as_string().map_or(
		manga_object.get("manga_cover_img").as_string()?.read(),
		|url| url.read(),
	);
	let title = manga_object.get("manga_name").as_string()?.read();

	Ok(Manga {
		id,
		cover,
		title,
		..Default::default()
	})
}

// Parse complete manga info from manga details object
pub fn parse_manga_details(manga_object: &ObjectRef) -> Result<Manga> {
	let basic_manga_object = manga_object.get("detail").as_object()?;
	let basic_manga = parse_basic_manga(&basic_manga_object)?;

	let description_text = basic_manga_object
		.get("manga_description")
		.as_string()?
		.read();
	// strip html tags from some descriptions
	let description = Node::new_fragment(&description_text)
		.map(|node| node.text().read())
		.unwrap_or(description_text);

	let url = parse_manga_url_id(&basic_manga.cover, &basic_manga.id)
		.map_or(String::default(), |url_id| {
			format!("{BASE_URL}/{}c{}", url_id, basic_manga.id)
		});

	let author = manga_object
		.get("authors")
		.as_array()?
		.filter_map(|author| {
			author
				.as_object()
				.and_then(|author| author.get("author_name").as_string())
				.map(|author| author.read())
				.ok()
		})
		.collect::<Vec<String>>()
		.join(", ");

	let categories = manga_object
		.get("tags")
		.as_array()?
		.filter_map(|tag| {
			tag.as_object()
				.and_then(|tag| tag.get("tag_name").as_string())
				.map(|tag| tag.read())
				.ok()
		})
		.collect::<Vec<String>>();

	let status = match basic_manga_object
		.get("manga_status")
		.as_bool()
		.unwrap_or(false)
	{
		true => MangaStatus::Completed,
		false => MangaStatus::Ongoing,
	};

	let mut nsfw = MangaContentRating::Safe;
	for category in &categories {
		match category.to_lowercase().as_str() {
			"adult" | "mature" | "smut" => {
				nsfw = MangaContentRating::Nsfw;
				break;
			}
			"ecchi" => nsfw = MangaContentRating::Suggestive,
			_ => (),
		};
	}

	Ok(Manga {
		author,
		description,
		url,
		categories,
		status,
		nsfw,
		viewer: MangaViewer::Rtl, // there are "manhwa" and "manhua" categories, but nothing in there so we can ignore for now
		..basic_manga
	})
}

pub fn parse_chapters(manga_object: &ObjectRef) -> Result<Vec<Chapter>> {
	let url_prefix = {
		let basic_manga = parse_basic_manga(&manga_object.get("detail").as_object()?)?;
		parse_manga_url_id(&basic_manga.cover, &basic_manga.id)
			.map_or(String::default(), |url_id| {
				format!("{BASE_URL}/read/{}c{}/chapter-", url_id, &basic_manga.id)
			})
	};
	Ok(manga_object
		.get("chapters")
		.as_array()?
		.filter_map(|chapter_obj| {
			let chapter_obj = chapter_obj.as_object().ok()?;
			let chapter = chapter_obj.get("chapter_number").as_float().unwrap_or(-1.0) as f32;
			// use chapter number for id instead of the chapter_id property, since that's what the api url takes
			let id = chapter.to_string();
			// let id = chapter_obj.get("chapter_id").as_string().ok()?.read();
			let title = chapter_obj.get("chapter_title").as_string().ok()?.read();
			let date_updated = chapter_obj
				.get("chapter_date_published")
				.as_date("yyyy-MM-dd'T'HH:mm:ss.SSS'Z'", Some("en-US"), Some("UTC"))
				.ok()?;
			let url = format!("{url_prefix}{chapter}");

			Some(Chapter {
				id,
				title,
				chapter,
				date_updated,
				url,
				lang: String::from("ja"),
				..Default::default()
			})
		})
		.collect::<Vec<_>>())
}

pub fn parse_pages(chapter_object: &ObjectRef) -> Result<Vec<Page>> {
	let detail = chapter_object.get("chapter_detail").as_object()?;
	let server = detail
		.get("server")
		.as_string()
		.map(|s| s.read())
		.unwrap_or(format!("{PAGE_URL}/"));
	let content = detail.get("chapter_content").as_string()?.read();
	let node = Node::new_fragment_with_uri(content, PAGE_URL)?;
	Ok(node
		.select("div.chapter-img canvas")
		.array()
		.enumerate()
		.map(|(index, node)| {
			let node = node.as_node().expect("array items should always be nodes");
			let src = node.attr("data-srcset").read();
			let url = if src.starts_with("http") {
				src
			} else {
				format!("{server}{src}")
			};
			Page {
				index: index as i32,
				url,
				..Default::default()
			}
		})
		.collect::<Vec<_>>())
}
