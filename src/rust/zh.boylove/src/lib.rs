#![no_std]

extern crate alloc;
mod helper;

use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	helpers::substring::Substring,
	prelude::*,
	std::{
		defaults::defaults_get, html::unescape_html_entities, json, net::Request, String, ValueRef,
		Vec,
	},
	Chapter, DeepLink, Filter, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	Page,
};
use alloc::{borrow::ToOwned as _, string::ToString};
use base64::{engine::general_purpose, Engine};
use helper::{
	setting::change_charset,
	url::{DefaultRequest as _, Index, LastUpdatedQuery, Url, CHAPTER_PATH, MANGA_PATH},
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
			let index = Index { page };

			Url::Uncensored { index }
				.get()
				.json()?
				.as_object()?
				.get("result")
				.as_object()?
				.get_manga_page_res()
		}

		"最新" => {
			let query = LastUpdatedQuery { page };
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

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let chapter_html = Url::Chapter(&chapter_id).get().html()?;

	let mut pages = Vec::<Page>::new();
	let page_nodes = chapter_html.select("img.lazy[id]");
	for (page_index, page_value) in page_nodes.array().enumerate() {
		let mut page_path = page_value
			.as_node()?
			.attr("data-original")
			.read()
			.trim()
			.to_string();
		if let Some(caps) = Regex::new(
			r"(?<chapter>.+[^a-z0-9])(?<page_id>[a-z0-9]{32,})\.(?<file_extension>[^\?]+)",
		)
		.expect("Invalid regular expression")
		.captures(&page_path)
		{
			let chapter = &caps["chapter"];
			let page_id = &caps["page_id"][..32];
			let file_extension = &caps["file_extension"];
			page_path = format!("{chapter}{page_id}.{file_extension}");
		};

		let page_url = Url::Abs { path: &page_path }.to_string();

		pages.push(Page {
			index: page_index as i32,
			url: page_url,
			..Default::default()
		});
	}

	Ok(pages)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.default_headers();
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	if url.contains(MANGA_PATH) {
		let Some(manga_id) = url.substring_after_last("/") else {
			return Ok(DeepLink::default());
		};
		let manga = Some(get_manga_details(manga_id.to_string())?);

		return Ok(DeepLink {
			manga,
			chapter: None,
		});
	}

	if !url.contains(CHAPTER_PATH) {
		return Ok(DeepLink::default());
	}

	let Some(chapter_id) = url.substring_after_last("/") else {
		return Ok(DeepLink::default());
	};
	let chapter = Some(Chapter {
		id: chapter_id.to_string(),
		..Default::default()
	});

	let chapter_html = Url::Chapter(chapter_id).get().html()?;
	let manga_url = chapter_html
		.select("a.icon-only.link.back")
		.attr("href")
		.read();
	let Some(manga_id) = manga_url.substring_after_last("/") else {
		return Ok(DeepLink {
			manga: None,
			chapter,
		});
	};
	let manga = Some(get_manga_details(manga_id.to_string())?);

	Ok(DeepLink { manga, chapter })
}

#[handle_notification]
fn handle_notification(notification: String) {
	match notification.as_str() {
		"changeCharset" => change_charset(),
		"signIn" => sign_in().unwrap_or_default(),
		_ => (),
	}
}

/// Returns [`Safe`](MangaContentRating::Safe) if the given slice contains
/// `清水`, or else returns [`Nsfw`](MangaContentRating::Nsfw).
fn get_content_rating(categories: &[String]) -> MangaContentRating {
	if categories.contains(&"清水".to_string()) {
		return MangaContentRating::Safe;
	}
	MangaContentRating::Nsfw
}

fn sign_in() -> Result<()> {
	let captcha = defaults_get("captcha")?.as_string()?.read();

	let is_wrong_captcha_format = captcha.parse::<u16>().is_err() || captcha.chars().count() != 4;
	if is_wrong_captcha_format {
		let sign_in_page = Url::SignInPage.get().html()?;

		let captcha_img_path = sign_in_page.select("img#verifyImg").attr("src").read();
		let captcha_img = Url::Abs {
			path: &captcha_img_path,
		}
		.get()
		.data();
		let base64_img = general_purpose::STANDARD_NO_PAD.encode(captcha_img);

		return Ok(println!("{}", base64_img));
	}

	let username = defaults_get("username")?.as_string()?.read();
	let password = defaults_get("password")?.as_string()?.read();
	let sign_in_data = format!(
		"username={}&password={}&vfycode={}&type=login",
		username, password, captcha
	);

	let response_json = Url::SignIn.post(sign_in_data).json()?;
	let reponse_obj = response_json.as_object()?;
	let info = reponse_obj.get("info").as_string()?;

	Ok(println!("{}", info))
}

trait Parser {
	/// Returns [`None`], or the text of the Node (if [`Ok`]).
	fn get_is_ok_text(self) -> Option<String>;
}

impl Parser for ValueRef {
	fn get_is_ok_text(self) -> Option<String> {
		self.as_node().map(|node| node.text().read()).ok()
	}
}
