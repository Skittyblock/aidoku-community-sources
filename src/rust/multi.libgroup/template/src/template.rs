use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	helpers::uri::QueryParameters,
	prelude::format,
	std::{
		current_date,
		defaults::defaults_get,
		net::Request,
		String, Vec,
	},
	Chapter, Filter, Listing, Manga, MangaContentRating, MangaPageResult, Page,
};
use alloc::string::ToString;
extern crate alloc;

use crate::{
	helpers::{get_token, is_logged, save_token},
	parser,
};

pub struct SocialLibSource {
	pub site_id: &'static str,
	pub domain: &'static str,
	pub nsfw: &'static MangaContentRating,
}

pub struct CDN {
	pub main: String,
	pub second: String,
	pub compress: String,
}

static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1";
static DOMAIN_API: &str = "https://api2.mangalib.me/api/";

impl SocialLibSource {
	pub fn refresh_token(&self) -> Result<()> {
		let timestamp = defaults_get("timestamp")?.as_int()? / 1000;
		let access_token = defaults_get("access_token")?.as_string()?.read();
		let refresh_token = defaults_get("refresh_token")?.as_string()?.read();
		let now = current_date() as i64;
		let expires_in = defaults_get("expires_in")?.as_int()?;
		if (now - timestamp) >= expires_in {
			let url = format!("{}auth/oauth/token", DOMAIN_API);
			let auth = format!("Bearer {}", access_token);
			let domain = format!("https://{}/", self.domain);
			let body = format!(
				r#"{{"grant_type":"refresh_token","client_id":"{}","refresh_token":"{}","scope":""}}"#,
				self.site_id, refresh_token
			);

			let request = Request::post(url)
				.header("User-Agent", USER_AGENT)
				.header("Site-Id", self.site_id)
				.header("Content-Type", "application/json")
				.header("Authorization", auth.as_str())
				.header("Referer", domain.as_str())
				.body(body);
			let json = request.json()?.as_object()?;

			save_token(json);
			return Ok(());
		}
		Ok(())
	}

	fn request_get(&self, url: &str) -> Request {
		let req = Request::get(url)
			.header("Site-Id", self.site_id)
			.header("User-Agent", USER_AGENT);
		if is_logged() {
			req.header("authorization", get_token().as_str())
		} else {
			req
		}
	}

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
		let request = self.request_get(&url);

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
			let request = self.request_get(&url);
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
		let request = self.request_get(&url);
		let json = request.json()?.as_object()?;

		parser::parse_manga_details(json, self.domain, self.nsfw)
	}

	pub fn get_chapter_list(&self, id: String) -> Result<Vec<Chapter>> {
		let url = format!("{}manga/{}/chapters", DOMAIN_API, id);

		let request = self.request_get(&url);
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
		let request = self.request_get(&url);
		let json = request.json()?.as_object()?;
		let cdn = self.get_cdn_domains()?;

		parser::parse_page_list(json, &cdn)
	}

	pub fn modify_image_request(&self, request: Request) {
		request.header("Referer", &format!("https://{}", self.domain));
	}

	pub fn get_cdn_domains(&self) -> Result<CDN> {
		let url = format!("{}constants?fields[]=imageServers", DOMAIN_API);
		let request = Request::get(url)
			.header("Site-Id", self.site_id)
			.header("User-Agent", USER_AGENT);
		let json = request.json()?.as_object()?;

		let site_id = self.site_id.parse::<i64>().map_err(|_| AidokuError {
			reason: AidokuErrorKind::ValueCast(aidoku::error::ValueCastError::NotInt),
		})?;
		parser::parse_image_servers_list(json, site_id)
	}
}
