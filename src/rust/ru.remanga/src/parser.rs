use crate::dto::{BaseMangaItem, ChapterContainer, FetchMangaInfo};
use crate::helper::{
	build_url_to_chapter, build_url_to_cover, build_url_to_title, get_manga_title,
};
use aidoku::helpers::node::NodeHelpers;
use aidoku::std::html::Node;
use aidoku::std::ArrayRef;
use aidoku::{
	error::Result, std::ObjectRef, Chapter, Manga, MangaContentRating, MangaPageResult,
	MangaStatus, MangaViewer, Page,
};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub fn parse_manga_fetch_info(id: String) -> Result<FetchMangaInfo> {
	if let Some((branches_raw, dir)) = id.split_once(':') {
		Ok(FetchMangaInfo {
			dir: String::from(dir),
			branches: branches_raw.split(',').map(String::from).collect(),
		})
	} else {
		Ok(FetchMangaInfo {
			dir: id,
			branches: Vec::new(),
		})
	}
}

fn parse_cover_image(manga_obj: &ObjectRef) -> Result<String> {
	let cover_obj = manga_obj.get("cover").as_object()?;
	cover_obj
		.get("high")
		.as_string()
		.map(|x| x.read())
		.or_else(|_| {
			cover_obj
				.get("mid")
				.as_string()
				.map(|x| x.read())
				.or_else(|_| cover_obj.get("low").as_string().map(|x| x.read()))
		})
}

fn parse_manga_base(manga_obj: &ObjectRef) -> Result<BaseMangaItem> {
	let dir = manga_obj.get("dir").as_string()?.read();
	let main_name = manga_obj.get("main_name").as_string()?.read();
	let secondary_name = manga_obj
		.get("secondary_name")
		.as_string()
		.map(|r| r.read())
		.unwrap_or(main_name.clone());
	let cover = parse_cover_image(manga_obj)
		.map(build_url_to_cover)
		.unwrap_or_default();
	let status_id = manga_obj
		.get("status")
		.as_object()?
		.get("id")
		.as_int()
		.unwrap_or(-1);
	let status = match status_id {
		1 => MangaStatus::Completed,
		2 => MangaStatus::Ongoing,
		3 => MangaStatus::Hiatus,
		_ => MangaStatus::Unknown,
	};
	let viewer = manga_obj
		.get("type")
		.as_object()?
		.get("id")
		.as_int()
		.map(|type_id| match type_id {
			1 => MangaViewer::Rtl,
			// idk but seems that most of non-manga titles looks fine in scroll viewer...
			_ => MangaViewer::Scroll,
		})
		.unwrap_or(MangaViewer::Rtl);
	let url = build_url_to_title(&dir);

	Ok(BaseMangaItem {
		dir,
		main_name,
		secondary_name,
		cover,
		viewer,
		status,
		url,
	})
}

fn parse_creators(creators: ArrayRef) -> Result<String> {
	let mut result = Vec::new();
	for creator_val_ref in creators {
		let creator_obj = creator_val_ref.as_object()?;

		let creator_type = creator_obj.get("type").as_int().unwrap_or(-1);
		if creator_type != 1 || creator_type != 4 {
			continue;
		}

		let name = creator_obj.get("name").as_string()?.read();
		result.push(name);
	}
	Ok(result.join(", "))
}

fn parse_names_list(list: ArrayRef) -> Result<Vec<String>> {
	let mut result = Vec::new();
	for obj_ref in list {
		let obj = obj_ref.as_object()?;

		let name = obj.get("name").as_string()?.read();
		result.push(name);
	}

	Ok(result)
}

pub fn parse_branches(branches: ArrayRef) -> Result<String> {
	let mut result = Vec::new();
	for branch_ref in branches {
		let branch_obj = branch_ref.as_object()?;

		let id = branch_obj.get("id").as_int()?.to_string();
		result.push(id);
	}

	Ok(result.join(","))
}

pub fn parse_manga_item(manga_obj: ObjectRef) -> Result<Manga> {
	let manga_base = parse_manga_base(&manga_obj)?;

	let title = get_manga_title(&manga_base);
	let mut description = manga_obj
		.get("description")
		.as_string()
		.unwrap_or_default()
		.read();
	if !description.is_empty() {
		description = Node::new(description.clone()) // they have HTML tags that needs to be cleaned
			.map(|n| n.text_with_newlines())
			.unwrap_or(description)
			.trim()
			.to_string();
	}

	let rating = manga_obj
		.get("age_limit")
		.as_object()?
		.get("id")
		.as_int()
		.map(|age_type_id| match age_type_id {
			2 => MangaContentRating::Nsfw,
			1 => MangaContentRating::Suggestive,
			_ => MangaContentRating::Safe,
		})
		.unwrap_or(MangaContentRating::Safe);
	let author = manga_obj
		.get("creators")
		.as_array()
		.and_then(parse_creators)?;
	let mut categories = manga_obj
		.get("categories")
		.as_array()
		.and_then(parse_names_list)?;
	let mut genres = manga_obj
		.get("genres")
		.as_array()
		.and_then(parse_names_list)?;
	categories.append(&mut genres);
	let branches = manga_obj
		.get("branches")
		.as_array()
		.and_then(parse_branches)?;
	let id = format!("{}:{}", branches, manga_base.dir);

	Ok(Manga {
		id,
		cover: manga_base.cover.clone(),
		title,
		author,
		description,
		url: manga_base.url.clone(),
		categories,
		status: manga_base.status,
		nsfw: rating,
		viewer: manga_base.viewer,
		..Default::default()
	})
}

pub fn parse_manga_list(results: ArrayRef) -> Result<MangaPageResult> {
	let has_more = !results.is_empty();
	let mut manga = Vec::new();
	for result in results {
		let obj = result.as_object()?;
		let item = parse_manga_base(&obj)?;
		let title = get_manga_title(&item);

		manga.push(Manga {
			id: format!(":{}", item.dir),
			cover: item.cover,
			title,
			url: item.url,
			status: item.status,
			viewer: item.viewer,
			..Default::default()
		});
	}

	Ok(MangaPageResult { has_more, manga })
}

fn parse_chapter(dir: &String, chapter_obj: ObjectRef) -> Result<ChapterContainer> {
	let index = chapter_obj.get("index").as_int()?;

	let id = chapter_obj.get("id").as_int()?.to_string();
	let title = chapter_obj
		.get("name")
		.as_string()
		.unwrap_or_default()
		.read();
	let volume = chapter_obj.get("tome").as_int().unwrap_or(0) as f32;
	let chapter = chapter_obj
		.get("chapter")
		.as_string()?
		.read()
		.parse::<f32>()
		.unwrap_or(-1.0);
	let date_updated = chapter_obj
		.get("upload_date")
		.as_date("yyyy-MM-dd'T'HH:mm:ss.SSSXXX", Some("en_US"), None)
		.unwrap_or(-1.0);
	let scanlator = chapter_obj
		.get("publishers")
		.as_array()
		.and_then(parse_names_list)
		.map(|x| x.join(", "))
		.unwrap_or_default();
	let url = build_url_to_chapter(&id, dir);

	let item = Chapter {
		id,
		title,
		volume,
		chapter,
		date_updated,
		scanlator,
		url,
		lang: String::from("ru"),
	};

	Ok(ChapterContainer { index, item })
}

pub fn parse_chapters(dir: String, results: ArrayRef) -> Result<Vec<ChapterContainer>> {
	let mut chapters = Vec::new();

	for result in results {
		let obj = result.as_object()?;

		let is_published = obj.get("is_published").as_bool().unwrap_or(true);
		if !is_published {
			continue; // skip unpublished chapters
		}

		let is_paid = obj.get("is_paid").as_bool().unwrap_or(false);
		if is_paid {
			let is_bought = obj.get("is_bought").as_bool().unwrap_or(false);
			let is_free_today = obj.get("is_free_today").as_bool().unwrap_or(false);
			if !is_bought && !is_free_today {
				continue; // skip paid chapters that is not available to fetch rn
			}
		}

		let chapter = parse_chapter(&dir, obj)?;
		chapters.push(chapter);
	}

	Ok(chapters)
}

fn parse_page(obj: ObjectRef) -> Result<Page> {
	let index = obj.get("id").as_int()? as i32;
	let url = obj.get("link").as_string()?.read();

	Ok(Page {
		index,
		url,
		..Default::default()
	})
}

pub fn parse_pages(obj: ObjectRef) -> Result<Vec<Page>> {
	let mut pages = Vec::new();

	let results = obj.get("pages").as_array()?;
	for result in results {
		// i dunno why they return object pages[][]
		// i haven't seen more than one page in page here
		// maybe this is for injecting ads... anyway i don't know atm how to filter it
		// if so
		let pages_list = result.as_array()?;
		for v_page in pages_list {
			let page = v_page.as_object().and_then(parse_page)?;

			pages.push(page);
		}
	}

	Ok(pages)
}
