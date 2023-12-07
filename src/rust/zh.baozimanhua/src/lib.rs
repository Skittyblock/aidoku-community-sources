#![no_std]
extern crate alloc;
mod url;

use aidoku::{
	error::Result,
	helpers::substring::Substring,
	prelude::get_manga_list,
	std::{net::Request, Vec},
	Filter, Manga, MangaPageResult,
};
use alloc::string::ToString;
use url::Url;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let manga_list_url = Url::from((filters, page));

	if let Url::Filters(_) = manga_list_url {
		let filters_obj = Request::get(manga_list_url.to_string())
			.json()?
			.as_object()?;

		let manga = filters_obj
			.get("items")
			.as_array()?
			.map(|value| {
				let obj = value.as_object()?;

				let id = obj.get("comic_id").as_string()?.read();

				let cover = {
					let file_name = obj.get("topic_img").as_string()?.read();
					Url::Cover(&file_name).to_string()
				};

				let title = obj.get("name").as_string()?.read();

				let artist = {
					let mut artists = obj
						.get("author")
						.as_string()?
						.read()
						.split(',')
						.map(ToString::to_string)
						.collect::<Vec<_>>();
					artists.dedup();

					artists.join("、")
				};

				let url = Url::Manga(&id).to_string();

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

	let manga = Request::get(manga_list_url.to_string())
		.html()?
		.select("div.comics-card")
		.array()
		.map(|value| {
			let div = value.as_node()?;

			let url = div.select("a.comics-card__poster").attr("abs:href").read();

			let id = url
				.substring_after_last('/')
				.expect("Unable to get the substring after the last '/'")
				.to_string();

			let cover = {
				let resized_cover = div.select("amp-img[noloading]").attr("src").read();
				resized_cover
					.clone()
					.substring_before_last('?')
					.map_or(resized_cover, ToString::to_string)
			};

			let title = div.select("h3").text().read();

			let artist = {
				let mut artists = div
					.select("small")
					.text()
					.read()
					.split(',')
					.map(ToString::to_string)
					.collect::<Vec<_>>();
				artists.dedup();

				artists.join("、")
			};

			let categories = div
				.select("span")
				.array()
				.map(|value| {
					let genre = value.as_node()?.text().read();
					Ok(genre)
				})
				.collect::<Result<_>>()?;

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

	Ok(MangaPageResult {
		manga,
		has_more: false,
	})
}

// #[get_manga_listing]
// fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult>
// { 	todo!()
// }

// #[get_manga_details]
// fn get_manga_details(id: String) -> Result<Manga> {
// 	todo!()
// }

// #[get_chapter_list]
// fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
// 	todo!()
// }

// #[get_page_list]
// fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
// 	todo!()
// }

// #[modify_image_request]
// fn modify_image_request(request: Request) {
// 	todo!()
// }

// #[handle_url]
// fn handle_url(url: String) -> Result<DeepLink> {
// 	todo!()
// }
