use aidoku::{
	std::{String, Vec},
	Manga, MangaContentRating, MangaStatus,
};
use alloc::borrow::ToOwned;
use serde::Deserialize;

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct PageWrapperDto<T> {
	pub content: Vec<T>,
	pub empty: bool,
	pub first: bool,
	pub last: bool,
	pub number: i32,
	pub number_of_elements: i32,
	pub size: i32,
	pub total_elements: i64,
	pub total_pages: i32,
}

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct SeriesDto<'a> {
	pub id: &'a str,
	pub library_id: &'a str,
	pub name: String,
	pub created: Option<&'a str>,
	pub last_modified: Option<&'a str>,
	pub file_last_modified: &'a str,
	pub books_count: i32,
	pub metadata: SeriesMetadataDto<'a>,
	pub books_metadata: BookMetadataAggregationDto<'a>,
}

impl SeriesDto<'_> {
	pub fn into_manga<T: AsRef<str>>(self, base_url: T) -> Manga {
		let base_url = base_url.as_ref();
		Manga {
			url: [base_url, "/series/", self.id].concat(),
			cover: [base_url, "/api/v1/series/", self.id, "/thumbnail"].concat(),
			id: self.id.to_owned(),
			title: self.metadata.title,
			author: self
				.books_metadata
				.authors
				.iter()
				.filter_map(|a| {
					if a.role == "writer" {
						Some(a.name.clone())
					} else {
						None
					}
				})
				.collect(),
			artist: self
				.books_metadata
				.authors
				.iter()
				.filter_map(|a| {
					if a.role == "penciller" {
						Some(a.name.clone())
					} else {
						None
					}
				})
				.collect(),
			categories: [self.metadata.genres, self.metadata.tags].concat(),
			description: if self.metadata.summary.is_empty() {
				self.books_metadata.summary
			} else {
				self.metadata.summary
			},
			status: match self.metadata.status {
				"ENDED" => MangaStatus::Completed,
				"ONGOING" => MangaStatus::Ongoing,
				"ABANDONED" => MangaStatus::Cancelled,
				"HIATUS" => MangaStatus::Hiatus,
				_ => MangaStatus::Unknown,
			},
			nsfw: if self.metadata.age_rating.unwrap_or(0) >= 18 {
				MangaContentRating::Nsfw
			} else if self.metadata.age_rating.unwrap_or(0) >= 16 {
				MangaContentRating::Suggestive
			} else {
				MangaContentRating::Safe
			},
			viewer: match self.metadata.reading_direction {
				"LEFT_TO_RIGHT" => aidoku::MangaViewer::Ltr,
				"RIGHT_TO_LEFT" => aidoku::MangaViewer::Rtl,
				"VERTICAL" => aidoku::MangaViewer::Vertical,
				"SCROLL" | "WEBTOON" => aidoku::MangaViewer::Scroll,
				_ => aidoku::MangaViewer::Rtl,
			},
		}
	}
}

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct SeriesMetadataDto<'a> {
	pub status: &'a str,
	pub created: Option<&'a str>,
	pub last_modified: Option<&'a str>,
	pub title: String,
	pub title_sort: String,
	pub summary: String,
	pub summary_lock: bool,
	pub reading_direction: &'a str,
	pub reading_direction_lock: bool,
	pub publisher: String,
	pub publisher_lock: bool,
	pub age_rating: Option<i32>,
	pub age_rating_lock: bool,
	pub language: &'a str,
	pub language_lock: bool,
	pub genres: Vec<String>,
	pub genres_lock: bool,
	pub tags: Vec<String>,
	pub tags_lock: bool,
	pub total_book_count: Option<i32>,
}

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct AuthorDto {
	pub name: String,
	pub role: String,
}

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct BookMetadataAggregationDto<'a> {
	pub authors: Vec<AuthorDto>,
	pub tags: Vec<String>,
	pub release_date: Option<&'a str>,
	pub summary: String,
	pub summary_number: String,

	pub created: &'a str,
	pub last_modified: &'a str,
}

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct MediaDto {
	pub status: String,
	pub media_type: String,
	pub pages_count: i32,
}

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct BookMetadataDto {
	pub title: String,
	pub title_lock: bool,
	pub summary: String,
	pub summary_lock: bool,
	pub number: String,
	pub number_lock: bool,
	pub number_sort: f32,
	pub number_sort_lock: bool,
	pub release_date: Option<String>,
	pub release_date_lock: bool,
	pub authors: Vec<AuthorDto>,
	pub authors_lock: bool,
}

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct BookDto<'a> {
	pub id: &'a str,
	pub series_id: &'a str,
	pub series_title: String,
	pub name: String,
	pub number: f32,
	pub created: Option<&'a str>,
	pub last_modified: Option<&'a str>,
	pub file_last_modified: &'a str,
	pub size_bytes: i64,
	pub size: &'a str,
	pub media: MediaDto,
	pub metadata: BookMetadataDto,
}

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct PageDto<'a> {
	pub number: i32,
	pub file_name: String,
	pub media_type: &'a str,
}
