#![no_std]

extern crate alloc;

pub mod parser;

use aidoku::{error::Result, prelude::*, Filter, helpers::uri::QueryParameters, std::{String, Vec}, std::net::{Request, HttpMethod}, MangaPageResult, Manga};
use aidoku::std::{ObjectRef, ValueRef};

static USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.127 Safari/537.36";

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
                    println!("Get token {}", token);
                    CSRF_TOKEN = Some(String::from(&token));
                    Ok(token)
                }
                Some(token) => Ok(String::from(token))
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

    pub fn get_manga_list_request(&self, filters: Vec<Filter>, page: i32) -> Result<Request> {
        let mut buffer = itoa::Buffer::new();
        let mut parameters = QueryParameters::new();
        parameters.push("page", Some(buffer.format(page)));
        let url = format!("{}/filterlist?{}", self.base_url, parameters);
        println!("Mangalib list url {}", url);
        self.base_request(url, HttpMethod::Post)
    }
}