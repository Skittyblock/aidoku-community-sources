#![no_std]
extern crate alloc;
mod url;

use aidoku::{
	error::Result,
	helpers::substring::Substring,
	prelude::*,
	std::{
		defaults::defaults_get,
		html::unescape_html_entities,
		net::{HttpMethod, Request},
		String, ValueRef, Vec,
	},
	Chapter, DeepLink, Filter, Manga, MangaContentRating, MangaPageResult, MangaStatus, Page,
};
use alloc::string::ToString;
use base64::{engine::general_purpose, Engine};
use regex::Regex;
use url::{Url, CHAPTER_PATH, DOMAIN, MANGA_PATH, USER_AGENT};

#[initialize]
fn initialize() {
	switch_chinese_char_set();
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	if page == 1 {
		check_in();
	}

	let manga_list_json = Url::from(filters, page)?.request(HttpMethod::Get).json()?;
	let manga_list_obj = manga_list_json.as_object()?;
	let result = manga_list_obj.get("result").as_object()?;

	let mut manga = Vec::<Manga>::new();
	let manga_arr = result.get("list").as_array()?;
	for manga_value in manga_arr {
		let manga_obj = manga_value.as_object()?;
		let keyword = manga_obj.get("keyword").as_string()?.read();

		// !! There's an ad whose lanmu_id is not 5, DO NOT use
		// // let is_ad = manga_obj.get("lanmu_id").as_int().unwrap_or(0) == 5;
		let is_ad = keyword.contains("公告");
		if is_ad {
			continue;
		}

		let manga_id = manga_obj.get("id").as_int()?.to_string();

		let cover_path = manga_obj.get("image").as_string()?.read();
		let cover_url = Url::Abs(cover_path).to_string();

		let manga_title = manga_obj.get("title").as_string()?.read();

		let artists_str = manga_obj
			.get("auther")
			.as_string()?
			.read()
			.replace('&', "、");

		let description = manga_obj.get("desc").as_string()?.read();

		let manga_url = Url::Manga(&manga_id).to_string();

		let categories = keyword
			.split(',')
			.filter(|tag| !tag.is_empty())
			.map(ToString::to_string)
			.collect::<Vec<String>>();

		let status = match manga_obj.get("mhstatus").as_int()? {
			0 => MangaStatus::Ongoing,
			1 => MangaStatus::Completed,
			_ => MangaStatus::Unknown,
		};

		let content_rating = get_content_rating(&categories);

		manga.push(Manga {
			id: manga_id,
			cover: cover_url,
			title: manga_title,
			author: artists_str.clone(),
			artist: artists_str,
			description,
			url: manga_url,
			categories,
			status,
			nsfw: content_rating,
			..Default::default()
		});
	}

	let has_more = !result.get("lastPage").as_bool()?;

	Ok(MangaPageResult { manga, has_more })
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let manga_html = Url::Manga(&manga_id).request(HttpMethod::Get).html()?;

	let cover_url = manga_html.select("a.play").attr("abs:data-original").read();

	let manga_title = manga_html.select("div.title > h1").text().read();

	let artists_str = manga_html
		.select("p.data:contains(作者：) > a")
		.array()
		.filter_map(Parser::get_is_ok_text)
		.collect::<Vec<String>>()
		.join("、");

	let mut description =
		unescape_html_entities(manga_html.select("span.detail-text").html().read())
			.split("<br>")
			.map(str::trim)
			.collect::<Vec<&str>>()
			.join("\n")
			.trim()
			.to_string();
	if let Some(description_removed_closing_tag) = description.substring_before_last("</") {
		description = description_removed_closing_tag.trim().to_string();
	}

	let manga_url = Url::Manga(&manga_id).to_string();

	let categories = manga_html
		.select("a.tag > span")
		.array()
		.filter_map(Parser::get_is_ok_text)
		.filter(|tag| !tag.is_empty())
		.collect::<Vec<String>>();

	let status = match manga_html.select("p.data").first().text().read().as_str() {
		"连载中" | "連載中" => MangaStatus::Ongoing,
		"完结" | "完結" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};

	let content_rating = get_content_rating(&categories);

	Ok(Manga {
		id: manga_id,
		cover: cover_url,
		title: manga_title,
		author: artists_str.clone(),
		artist: artists_str,
		description,
		url: manga_url,
		categories,
		status,
		nsfw: content_rating,
		..Default::default()
	})
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let chapter_list_json = Url::ChapterList(manga_id).request(HttpMethod::Get).json()?;
	let chapter_list_obj = chapter_list_json.as_object()?;
	let result = chapter_list_obj.get("result").as_object()?;

	let mut chapters = Vec::<Chapter>::new();
	let chapters_arr = result.get("list").as_array()?;
	for (chapter_index, chapter_value) in chapters_arr.rev().enumerate() {
		let chapter_obj = chapter_value.as_object()?;

		let chapter_id = chapter_obj.get("id").as_int()?.to_string();

		let chapter_title = chapter_obj.get("title").as_string()?.read();

		let chapter_num = (chapter_index + 1) as f32;

		let chapter_url = Url::Chapter(&chapter_id).to_string();

		let chapter = Chapter {
			id: chapter_id,
			title: chapter_title,
			chapter: chapter_num,
			url: chapter_url,
			lang: "zh".to_string(),
			..Default::default()
		};
		chapters.insert(0, chapter);
	}

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let chapter_html = Url::Chapter(&chapter_id).request(HttpMethod::Get).html()?;

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

		let page_url = Url::Abs(page_path).to_string();

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
	request
		.header("Referer", DOMAIN)
		.header("User-Agent", USER_AGENT);
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

	let chapter_html = Url::Chapter(chapter_id).request(HttpMethod::Get).html()?;
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
		"switchChineseCharSet" => switch_chinese_char_set(),
		"signIn" => sign_in().unwrap_or_default(),
		_ => (),
	}
}

fn switch_chinese_char_set() {
	let is_tc = defaults_get("isTC")
		.and_then(|value| value.as_bool())
		.unwrap_or(true);
	let char_set = if is_tc { "T" } else { "S" };

	Url::CharSet(char_set).request(HttpMethod::Get).send();
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
		let sign_in_page = Url::SignInPage.request(HttpMethod::Get).html()?;

		let captcha_img_path = sign_in_page.select("img#verifyImg").attr("src").read();
		let captcha_img = Url::Abs(captcha_img_path).request(HttpMethod::Get).data();
		let base64_img = general_purpose::STANDARD_NO_PAD.encode(captcha_img);

		return Ok(println!("{}", base64_img));
	}

	let username = defaults_get("username")?.as_string()?.read();
	let password = defaults_get("password")?.as_string()?.read();
	let sign_in_data = format!(
		"username={}&password={}&vfycode={}&type=login",
		username, password, captcha
	);

	let response_json = Url::SignIn
		.request(HttpMethod::Post)
		.body(sign_in_data)
		.json()?;
	let reponse_obj = response_json.as_object()?;
	let info = reponse_obj.get("info").as_string()?;

	Ok(println!("{}", info))
}

fn check_in() {
	let not_auto_check_in = !defaults_get("autoCheckIn")
		.and_then(|value| value.as_bool())
		.unwrap_or(false);
	if not_auto_check_in {
		return;
	}

	let check_in_data = "auto=false&td=&type=1";

	Url::CheckIn
		.request(HttpMethod::Post)
		.body(check_in_data)
		.send();
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
