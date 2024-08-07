use aidoku::{
	std::{String, Vec},
	Manga, MangaContentRating, MangaStatus, Page,
};
use alloc::{borrow::ToOwned, string::ToString};
use serde::Deserialize;

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct PageWrapperDto<T> {
	pub archives: Vec<T>,
	pub page: i32,
	pub limit: i32,
	pub total: i64,
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct ArchiveDto<'a> {
	pub id: i64,
	pub hash: &'a str,
	pub title: String,
	pub artists: Option<Vec<TaxonomyDto<'a>>>,
	pub circles: Option<Vec<TaxonomyDto<'a>>>,
	pub description: Option<&'a str>,
	pub tags: Option<Vec<TaxonomyDto<'a>>>,
	pub images: Vec<ImageDto<'a>>,
	pub thumbnail_url: &'a str,
}

impl ArchiveDto<'_> {
	pub fn into_manga<T: AsRef<str>>(self, base_url: T) -> Manga {
		let base_url = base_url.as_ref();
		Manga {
			url: [base_url, "/g/", &self.id.to_string()].concat(),
			cover: self.thumbnail_url.to_owned(),
			id: self.id.to_string(),
			title: self.title,
			author: self
				.artists
				.unwrap_or_default()
				.into_iter()
				.map(|a| a.name)
				.collect(),
			artist: self
				.circles
				.unwrap_or_default()
				.into_iter()
				.map(|a| a.name)
				.collect(),
			description: self.description.unwrap_or("").to_owned(),
			categories: self
				.tags
				.unwrap_or_default()
				.into_iter()
				.map(|a| a.name.to_owned())
				.collect(),
			status: MangaStatus::Completed,
			nsfw: MangaContentRating::Safe,
			viewer: aidoku::MangaViewer::Ltr,
		}
	}

	pub fn into_pages<T: AsRef<str>>(self, cdn_url: T) -> Vec<Page> {
		let cdn_url = cdn_url.as_ref();
		self.images
			.into_iter()
			.map(|v| Page {
				index: v.page_number - 1,
				url: [cdn_url, "/image/", self.hash, "/", v.filename].concat(),
				..Default::default()
			})
			.collect()
	}
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct TaxonomyDto<'a> {
	pub name: &'a str,
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct ImageDto<'a> {
	pub filename: &'a str,
	pub page_number: i32,
}

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ArchiveListDto<'a> {
	pub id: i64,
	pub hash: &'a str,
	pub title: String,
	pub artists: Option<Vec<TaxonomyDto<'a>>>,
	pub circles: Option<Vec<TaxonomyDto<'a>>>,
	pub description: Option<&'a str>,
	pub tags: Option<Vec<TaxonomyDto<'a>>>,
	pub thumbnail_url: &'a str,
}

impl ArchiveListDto<'_> {
	pub fn into_manga<T: AsRef<str>>(self, base_url: T) -> Manga {
		let base_url = base_url.as_ref();
		Manga {
			url: [base_url, "/g/", &self.id.to_string()].concat(),
			cover: self.thumbnail_url.to_owned(),
			id: self.id.to_string(),
			title: self.title,
			author: self
				.artists
				.unwrap_or_default()
				.into_iter()
				.map(|a| a.name)
				.collect(),
			artist: self
				.circles
				.unwrap_or_default()
				.into_iter()
				.map(|a| a.name)
				.collect(),
			description: self.description.unwrap_or("").to_owned(),
			categories: self
				.tags
				.unwrap_or_default()
				.into_iter()
				.map(|a| a.name.to_owned())
				.collect(),
			status: MangaStatus::Completed,
			nsfw: MangaContentRating::Safe,
			viewer: aidoku::MangaViewer::Rtl,
		}
	}
}
