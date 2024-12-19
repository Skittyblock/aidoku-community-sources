#![no_std]

extern crate alloc;

mod helper;

use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	helpers::substring::Substring as _,
	prelude::{
		get_chapter_list, get_manga_details, get_manga_list, get_manga_listing, get_page_list,
		handle_notification, handle_url, initialize, modify_image_request,
	},
	std::{html::unescape_html_entities, json, net::Request, String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	Page,
};
use alloc::{borrow::ToOwned as _, string::ToString as _};
use helper::{
	setting::change_charset,
	url::{Api, DefaultRequest as _, Index, LastUpdatedQuery, Url},
	MangaList as _, MangaListRes as _, Part, Regex,
};

#[initialize]
fn initialize() {
	change_charset();
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	Url::from((filters, page))
		.get()
		.json()?
		.as_object()?
		.get("result")
		.as_object()?
		.get_manga_page_res()
}

#[expect(clippy::needless_pass_by_value)]
#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	match listing.name.as_str() {
		"無碼專區" => {
			let index = Index::from_page(page);

			Url::Uncensored { index }
				.get()
				.json()?
				.as_object()?
				.get("result")
				.as_object()?
				.get_manga_page_res()
		}

		"最新" => {
			let query = LastUpdatedQuery::new(page);
			let manga = Url::LastUpdated { query }
				.get()
				.json()?
				.as_object()?
				.get("result")
				.as_array()?
				.get_manga_list()?;

			let has_more = !manga.is_empty();

			Ok(MangaPageResult { manga, has_more })
		}

		"排行榜" => {
			let chart_json = Url::Chart { page }
				.get()
				.html()?
				.html()
				.read()
				.substring_after("JSON.parse(\"")
				.and_then(|str| str.substring_before("\");"))
				.ok_or(AidokuError {
					reason: AidokuErrorKind::JsonParseError,
				})?
				.replace(r#"\""#, r#"""#)
				.replace(r"\\", r"\");

			json::parse(chart_json)?.as_object()?.get_manga_page_res()
		}

		"猜你喜歡" => {
			let manga = Url::Random
				.get()
				.json()?
				.as_object()?
				.get("data")
				.as_array()?
				.get_manga_list()?;

			let has_more = true;

			Ok(MangaPageResult { manga, has_more })
		}

		_ => Ok(MangaPageResult::default()),
	}
}

#[expect(clippy::needless_pass_by_value)]
#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = Url::Manga { id: &id };

	let manga_page = url.get().html()?;
	let cover = manga_page.select("div.book img").attr("abs:src").read();

	let title = manga_page.select("p.book-title").text().read();

	let author = manga_page
		.select("li.info a")
		.array()
		.filter_map(|val| {
			let author = val.as_node().ok()?.text().read();

			Some(author)
		})
		.collect::<Vec<_>>()
		.join("、");

	let description = {
		let html = manga_page.select("p.book-desc").html().read();
		let unescaped_html = unescape_html_entities(html);
		let desc = Regex::new(r"<br ?\/?>")?
			.split(&unescaped_html)
			.map(str::trim)
			.collect::<Vec<_>>()
			.join("\n");

		desc.clone()
			.substring_before("</")
			.map_or(desc, Into::into)
			.trim()
			.into()
	};

	let mut nsfw = MangaContentRating::Nsfw;
	let categories = manga_page
		.select("a.tag span.tag")
		.array()
		.filter_map(|val| {
			let tag = val.as_node().ok()?.text().read();

			if tag == "清水" {
				nsfw = MangaContentRating::Safe;
			}

			(!tag.is_empty()).then_some(tag)
		})
		.collect();

	let status = match manga_page.select("ul.pl-0 li:eq(1)").text().read().as_str() {
		"連載中" | "连载中" => MangaStatus::Ongoing,
		"完結" | "完结" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};

	Ok(Manga {
		id: id.clone(),
		cover,
		title,
		author,
		description,
		url: url.into(),
		categories,
		status,
		nsfw,
		..Default::default()
	})
}

#[expect(clippy::needless_pass_by_value)]
#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let chapters = Url::ChapterList { id: &manga_id }
		.get()
		.json()?
		.as_object()?
		.get("result")
		.as_object()?
		.get("list")
		.as_array()?
		.map(|val| {
			let item = val.as_object()?;
			let id = item.get("id").as_int()?.to_string();

			let title = item
				.get("title")
				.as_string()
				.unwrap_or_default()
				.read()
				.trim()
				.to_owned();

			let part = title.parse::<Part>().unwrap_or_default();
			let volume = part.volume;

			let chapter = part.chapter;

			let date_updated = item
				.get("create_time")
				.as_date("yyyy-MM-dd HH:mm:ss", None, None)
				.unwrap_or(-1.0);

			let url = Url::ChapterPage { id: &id }.into();

			let lang = "zh".into();

			Ok(Chapter {
				id,
				title,
				volume,
				chapter,
				date_updated,
				url,
				lang,
				..Default::default()
			})
		})
		.collect::<Result<_>>()?;

	Ok(chapters)
}

#[expect(clippy::needless_pass_by_value)]
#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let pages = Api::chapter(&chapter_id)
		.get()
		.html()?
		.select("img.lazy")
		.array()
		.enumerate()
		.map(|(i, val)| {
			#[expect(
				clippy::cast_possible_truncation,
				clippy::cast_possible_wrap,
				clippy::as_conversions
			)]
			let index = i as _;

			let path = &val
				.as_node()?
				.attr("data-original")
				.read()
				.trim()
				.to_owned();
			let url = Url::Abs { path }.into();

			Ok(Page {
				index,
				url,
				..Default::default()
			})
		})
		.collect::<Result<_>>()?;

	Ok(pages)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.default_headers();
}

#[expect(clippy::needless_pass_by_value)]
#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let Some(caps) =
		Regex::new(r"^https?:\/\/[^/]+\/home\/book\/(?<type>index|capter)\/id\/(?<id>\d+)$")?
			.captures(&url)
	else {
		return Ok(DeepLink::default());
	};

	let id = &caps["id"];
	if &caps["type"] == "index" {
		let manga = get_manga_details(id.into())?;

		let chapter = None;

		return Ok(DeepLink {
			manga: Some(manga),
			chapter,
		});
	}

	let manga = Url::ChapterPage { id }
		.get()
		.html()?
		.select("a.back")
		.attr("href")
		.read()
		.substring_after_last('/')
		.map(|manga_id| get_manga_details(manga_id.into()))
		.transpose()?;

	let chapter = Chapter {
		id: id.into(),
		..Default::default()
	};

	Ok(DeepLink {
		manga,
		chapter: Some(chapter),
	})
}

#[expect(clippy::needless_pass_by_value)]
#[handle_notification]
fn handle_notification(notification: String) {
	if notification == "changeCharset" {
		change_charset();
	}
}
