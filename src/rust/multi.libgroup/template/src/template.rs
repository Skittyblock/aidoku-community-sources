use aidoku::{
	error::{AidokuError, Result}, helpers::uri::QueryParameters, prelude::{format, println}, std::{net::{HttpMethod, Request}, Vec}, Filter, Listing, MangaPageResult
};
use alloc::string::ToString;
extern crate alloc;

use crate::{helpers::{route, SiteId}, parser};

pub struct SocialLibSource {
	pub site_id: &'static SiteId
}

static DOMAIN_API: &str = "https://api.lib.social/api/";

impl SocialLibSource {
	pub fn get_manga_list(&self, filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
		let mut qs = QueryParameters::new();
		qs.push("site_id[]", Some(&route(self.site_id)));
		qs.push("page", Some(&format!("{}", page)));
		let mut query = qs.to_string();
		let search_parameters =  crate::helpers::search(filters);
	
		if !search_parameters.is_empty() {
			query += &format!("&{}", &search_parameters)
		}
		
		let url = format!("{}manga?{}", DOMAIN_API, query);
		println!("{}", url);
		let request = Request::new(url, HttpMethod::Get);
		let json = request.json()?.as_object()?;
	
		parser::parse_manga_list(json, self.site_id)
	}
	
	pub fn get_manga_listing(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		if &listing.name == "Сейчас читают" {
			let mut qs = QueryParameters::new();
			qs.push("site_id[]", Some(&route(self.site_id)));
			qs.push("page", Some(&format!("{}", page)));
			qs.push("popularity", Some("0"));
			qs.push("time", Some("day"));
			let query = qs.to_string();
	
			let url = format!("{}media/top-views?{}", DOMAIN_API, query);
			println!("{}", url);
			let request = Request::new(url, HttpMethod::Get);
			let json = request.json()?.as_object()?;
	
			parser::parse_manga_list(json, self.site_id)
		} else {
			Err(AidokuError {
				reason: aidoku::error::AidokuErrorKind::Unimplemented,
			})
		}
	}
	/*
	pub fn get_manga_details(todo!()) -> Result<Manga> {
		todo!()
	}
	
	pub fn get_chapter_list(todo!()) -> Result<Vec<Chapter>> {
		todo!()
	}
	
	pub fn get_page_list(todo!()) -> Result<Vec<Page>> {
		todo!()
	}
	
	pub fn modify_image_request(todo!(), request: Request) {
		todo!()
	}
	
	pub fn handle_url(todo!()) -> Result<DeepLink> {
		todo!()
	}
	*/
}