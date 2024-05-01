use aidoku::{
	error::{AidokuError, Result},
	helpers::uri::QueryParameters,
	prelude::format,
	std::{
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, Filter, Listing, Manga, MangaContentRating, MangaPageResult, Page,
};
use alloc::string::ToString;
extern crate alloc;

use crate::parser;

pub struct SocialLibSource {
	pub site_id: &'static str,
	pub domain: &'static str,
	pub nsfw: &'static MangaContentRating,
	pub cdn: &'static CDN,
}

pub struct CDN {
	pub main: &'static str,
	pub second: &'static str,
	pub compress: &'static str,
}

static DOMAIN_API: &str = "https://api.lib.social/api/";

impl SocialLibSource {
	pub fn get_manga_list(&self, filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
		let mut qs = QueryParameters::new();
		qs.push("site_id[]", Some(self.site_id));
		qs.push("page", Some(&format!("{}", page)));
		let mut query = qs.to_string();
		let search_parameters = crate::helpers::search(filters);

		if !search_parameters.is_empty() {
			query += &format!("&{}", &search_parameters)
		}

		let url = format!("{}manga?{}", DOMAIN_API, query);
		let request = Request::new(url, HttpMethod::Get).header("Site-Id", self.site_id);
		let json = request.json()?.as_object()?;

		parser::parse_manga_list(json, &self.domain.to_string(), self.nsfw)
	}

	pub fn get_manga_listing(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		if &listing.name == "Сейчас читают" {
			let mut qs = QueryParameters::new();
			qs.push("page", Some(&format!("{}", page)));
			qs.push("popularity", Some("1"));
			qs.push("time", Some("day"));
			let query = qs.to_string();

			let url = format!("{}media/top-views?{}", DOMAIN_API, query);
			let request = Request::new(url, HttpMethod::Get).header("Site-Id", self.site_id);
			let json = request.json()?.as_object()?;

			parser::parse_manga_list(json, &self.domain.to_string(), self.nsfw)
		} else {
			Err(AidokuError {
				reason: aidoku::error::AidokuErrorKind::Unimplemented,
			})
		}
	}

	pub fn get_manga_details(&self, id: String) -> Result<Manga> {
		let mut query = QueryParameters::new();
		query.push("fields[]", Some("eng_name"));
		query.push("fields[]", Some("summary"));
		query.push("fields[]", Some("genres"));
		query.push("fields[]", Some("authors"));
		query.push("fields[]", Some("manga_status_id"));
		query.push("fields[]", Some("status_id"));
		query.push("fields[]", Some("artists"));
		let url = format!("{}manga/{}?{}", DOMAIN_API, id, query.to_string());
		let request = Request::new(url, HttpMethod::Get).header("Site-Id", self.site_id);
		let json = request.json()?.as_object()?;

		parser::parse_manga_details(json, self.domain, self.nsfw)
	}

	pub fn get_chapter_list(&self, id: String) -> Result<Vec<Chapter>> {
		let url = format!("{}manga/{}/chapters", DOMAIN_API, id);

		let request = Request::new(url, HttpMethod::Get).header("Site-Id", self.site_id);
		let json = request.json()?.as_object()?;

		parser::parse_chapter_list(json, &id, self.domain)
	}

	pub fn get_page_list(&self, manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
		let numbers: Vec<&str> = chapter_id.split('#').collect::<Vec<&str>>();

		let url = format!(
			"{}manga/{}/chapter?number={}&volume={}",
			DOMAIN_API,
			manga_id,
			numbers.first().unwrap(),
			numbers.get(1).unwrap()
		);
		let request = Request::new(url, HttpMethod::Get).header("Site-Id", self.site_id);
		let json = request.json()?.as_object()?;

		parser::parse_page_list(json, self.cdn)
	}
}
