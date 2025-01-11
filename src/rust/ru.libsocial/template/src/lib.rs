#![no_std]

extern crate alloc;

pub mod parser;

use aidoku::std::defaults::defaults_get;
use aidoku::{
	error::Result,
	helpers::uri::QueryParameters,
	prelude::*,
	std::net::{HttpMethod, Request},
	std::String,
};

static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.127 Safari/537.36";

pub static mut CSRF_TOKEN: Option<String> = None;

pub struct LibGroup {
	pub base_url: &'static str,
}

impl LibGroup {
	pub fn token(&self) -> Result<String> {
		unsafe {
			match &CSRF_TOKEN {
				None => {
					let node = Request::get(self.base_url).html()?;
					let meta = node.select("meta[name=_token]");
					let token = meta.attr("content").read();
					CSRF_TOKEN = Some(String::from(&token));
					Ok(token)
				}
				Some(token) => Ok(String::from(token)),
			}
		}
	}

	pub fn base_request<T: AsRef<str>>(&self, url: T, method: HttpMethod) -> Result<Request> {
		Ok(Request::new(url, method)
			.header("User-Agent", USER_AGENT)
			.header("Accept", "application/json, text/plain, */*")
			.header("X-Requested-With", "XMLHttpRequest")
			.header("x-csrf-token", &self.token()?))
	}

	pub fn get_manga_list_request(&self, params: QueryParameters) -> Result<Request> {
		let url = format!("{}/filterlist?{}", self.base_url, params);
		self.base_request(url, HttpMethod::Post)
	}

	pub fn auth(&self) -> Result<()> {
		let login = defaults_get("email")?.as_string()?.read();
		let password = defaults_get("password")?.as_string()?.read();
		let mut params = QueryParameters::new();
		let token = Request::get("https://lib.social/login")
			.html()?
			.select("input[name=_token]")
			.attr("value")
			.read();
		params.push("_token", Some(&token));
		params.push("from", Some("https://lib.social/?section=home-updates"));
		params.push("email", Some(&login));
		params.push("password", Some(&password));
		params.push("remember", Some("on"));
		let request = Request::post("https://lib.social/login").body(format!("{}", params));
		request.send();
		let cookie = request
			.get_header("set-cookie")
			.map(|c| c.read())
			.unwrap_or(String::from("none"));
		todo!("Get redirect cookie and save token");
	}
}
