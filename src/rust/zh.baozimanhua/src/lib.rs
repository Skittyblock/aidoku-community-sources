#![no_std]
mod parser;
mod url;

use aidoku::{
	error::Result,
	helpers::{substring::Substring, uri::QueryParameters},
	prelude::{
		format, get_chapter_list, get_manga_details, get_manga_list, get_manga_listing,
		get_page_list, handle_url,
	},
	std::{net::Request, String, ValueRef, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, MangaStatus, Page,
};
use chinese_number::{ChineseCountMethod, ChineseToNumber};
use parser::{Artists, DivComicsCard};
use regex::Regex;
use url::Url;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let manga_list_url = Url::from((filters, page));
	if let Url::Filters(_) = manga_list_url {
		let filters_obj = manga_list_url.get().json()?.as_object()?;
		let manga = filters_obj
			.get("items")
			.as_array()?
			.map(|value| {
				let obj = value.as_object()?;
				let id = obj.get("comic_id").as_string()?.read();

				let cover = {
					let topic_img = obj.get("topic_img").as_string()?.read();

					Url::Cover(&topic_img).into()
				};

				let title = obj.get("name").as_string()?.read();

				let artist = obj.get("author").as_string()?.read().dedup_and_join();

				let url = Url::Manga(&id).into();

				let mut categories = obj
					.get("type_names")
					.as_array()?
					.filter_map(|value| {
						let genre = value.as_string().ok()?.read();

						(!genre.is_ascii()).then_some(genre)
					})
					.collect::<Vec<_>>();
				{
					let region = obj
						.get("region_name")
						.as_string()
						.or_else(|_| obj.get("region").as_string())?
						.read();
					categories.insert(0, region);
				}

				Ok(Manga {
					id,
					cover,
					title,
					author: artist.clone(),
					artist,
					url,
					categories,
					..Default::default()
				})
			})
			.collect::<Result<_>>()?;

		let has_more = filters_obj.get("next").as_string().is_ok();

		return Ok(MangaPageResult { manga, has_more });
	}

	let manga = manga_list_url.get().html()?.get_manga_list()?;

	Ok(MangaPageResult {
		manga,
		has_more: false,
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, _: i32) -> Result<MangaPageResult> {
	if listing.name == "最新上架" {
		let manga = Url::New.get().html()?.get_manga_list()?;

		return Ok(MangaPageResult {
			manga,
			has_more: false,
		});
	}

	let manga = {
		let selector = {
			let regex = match listing.name.as_str() {
				"熱門漫畫" => "熱門漫畫|热门漫画",
				"推薦中港台漫" => "推薦國漫|推荐国漫",
				"推薦韓漫" => "推薦韓漫|推荐韩漫",
				"推薦日漫" => "推薦日漫|推荐日漫",
				"熱血漫畫" => "熱血漫畫|热血漫画",
				"最近更新" => "最近更新",
				_ => return Ok(MangaPageResult::default()),
			};

			format!(
				"div.index-recommend-items:has(div.catalog-title:matches({}))",
				regex
			)
		};

		Url::Domain.get().html()?.select(selector).get_manga_list()
	}?;

	Ok(MangaPageResult {
		manga,
		has_more: false,
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = Url::Manga(&id).into();

	let manga_page = Request::get(&url).html()?;
	let cover = {
		let resized_cover = manga_page
			.select("meta[name=og:image]")
			.attr("content")
			.read();

		resized_cover
			.clone()
			.substring_before_last('?')
			.map_or(resized_cover, Into::into)
	};

	let title = manga_page
		.select("meta[name=og:novel:book_name]")
		.attr("content")
		.read();

	let artist = manga_page
		.select("meta[name=og:novel:author]")
		.attr("content")
		.read()
		.dedup_and_join();

	let description = {
		let og_description = manga_page
			.select("meta[name=og:description]")
			.attr("content")
			.read();

		og_description
			.clone()
			.substring_after("》全集,")
			.map_or(og_description, Into::into)
	};

	let categories = manga_page
		.select("span.tag:gt(0)")
		.array()
		.filter_map(|value| {
			let tag = value.as_node().ok()?.text().read();

			(!tag.is_empty()).then_some(tag)
		})
		.collect();

	let status = match manga_page
		.select("meta[name=og:novel:status]")
		.attr("content")
		.read()
		.as_str()
	{
		"連載中" | "连载中" => MangaStatus::Ongoing,
		"已完結" | "已完结" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};

	Ok(Manga {
		id,
		cover,
		title,
		author: artist.clone(),
		artist,
		description,
		url,
		categories,
		status,
		..Default::default()
	})
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let get_res_chapter = |value: ValueRef| {
		let a = value.as_node()?;
		let url = a.attr("abs:href").read();

		let id = url
			.substring_after_last('=')
			.expect("Unable to get the substring after the last '/'")
			.into();

		let title = a.text().read();

		let (volume, chapter) = {
			let get_volume_and_chapter_from = |title: &str| {
				let to_f32 = |str: &str| {
					str.parse().map_or_else(
						|_| str.to_number(ChineseCountMethod::TenThousand).ok(),
						Some,
					)
				};

				let Some(caps) = {
					let pat = concat!(
						r"^(\[?第?(?<volume>[\d零一二三四五六七八九十百千]+)([-+＋][\d零一二三四五六七八九十百千]+)?",
						r"[卷季部](\((?<part>[上下])\))?\]?)?\s?(\[?第?(?<chapter>[\d零一二三四五六七八九十百千]+(\.\d+)?)",
						r"([-+＋][\d零一二三四五六七八九十百千]+)?[話话回]?\]?)?"
					);

					Regex::new(pat)
				}
				.expect("Invalid regex")
				.captures(title) else {
					return (-1.0, -1.0);
				};
				let volume = {
					let part = caps
						.name("part")
						.and_then(|m| (m.as_str() == "下").then_some(0.5))
						.unwrap_or(0.0);
					caps.name("volume")
						.and_then(|m| {
							let num = to_f32(m.as_str())? + part;

							Some(num)
						})
						.unwrap_or(-1.0)
				};

				let chapter = caps
					.name("chapter")
					.and_then(|m| to_f32(m.as_str()))
					.unwrap_or(-1.0);

				(volume, chapter)
			};

			get_volume_and_chapter_from(&title)
		};

		Ok(Chapter {
			id,
			title,
			volume,
			chapter,
			url,
			lang: "zh".into(),
			..Default::default()
		})
	};

	let manga_page = Url::Manga(&manga_id).get().html()?;
	let chapters = manga_page
		.select("div.pure-g[id] a.comics-chapters__item")
		.array()
		.rev()
		.map(get_res_chapter)
		.collect::<Result<Vec<_>>>()?;

	if !chapters.is_empty() {
		return Ok(chapters);
	}

	manga_page
		.select("a.comics-chapters__item")
		.array()
		.map(get_res_chapter)
		.collect()
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut chapter_url = {
		let mut query = QueryParameters::new();
		query.push_encoded("comic_id", Some(&manga_id));
		query.push_encoded("section_slot", Some("0"));
		query.push_encoded("chapter_slot", Some(&chapter_id));

		Url::Chapter(query).into()
	};
	let mut pages = Vec::new();
	{
		let mut index = -1;
		loop {
			let chapter_page = Request::get(&chapter_url).html()?;
			for value in chapter_page.select("amp-img.comic-contain__item").array() {
				index += 1;

				let url = value.as_node()?.attr("data-src").read();

				pages.push(Page {
					index,
					url,
					..Default::default()
				});
			}

			chapter_url = chapter_page
				.select("a#next-chapter:has(i.icon-xiangxia)")
				.attr("href")
				.read();

			if chapter_url.is_empty() {
				break;
			}
		}
	}

	pages.sort_by_key(|page| page.url.clone());
	pages.dedup_by_key(|page| page.url.clone());
	pages.sort_by_key(|page| page.index);
	pages = pages
		.iter()
		.enumerate()
		.map(|(index, page)| Page {
			index: index as i32,
			..page.clone()
		})
		.collect();

	Ok(pages)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let Some(caps) = Regex::new(
		r"\/comic(\/chapter)?\/(?<manga_id>[^/]+)(\/0_(?<chapter_id>\d+)(_\d+)?\.html)?",
	)
	.expect("Invalid regex")
	.captures(&url) else {
		return Ok(DeepLink::default());
	};
	let manga = get_manga_details(caps["manga_id"].into())?;

	let chapter = caps.name("chapter_id").map(|m| Chapter {
		id: m.as_str().into(),
		..Default::default()
	});

	Ok(DeepLink {
		manga: Some(manga),
		chapter,
	})
}
