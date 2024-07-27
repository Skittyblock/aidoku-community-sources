#![no_std]

mod helper;
mod parser;

use aidoku::{
	error::Result,
	helpers::uri::encode_uri_component,
	prelude::*,
	std::defaults::defaults_get,
	std::net::{HttpMethod, Request},
	std::{String, Vec},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaPageResult, Page,
};

use helper::*;
use parser::*;

const BASE_URL: &str = "https://asuracomic.net";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	todo!()
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	todo!()
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	todo!()
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	todo!()
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	todo!()
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	todo!()
}
