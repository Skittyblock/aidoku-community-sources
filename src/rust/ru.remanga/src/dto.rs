use aidoku::{Chapter, MangaStatus, MangaViewer};
use alloc::string::String;
use alloc::vec::Vec;

pub struct BaseMangaItem {
	pub dir: String,
	pub main_name: String,
	pub secondary_name: String,
	pub cover: String,
	pub viewer: MangaViewer,
	pub status: MangaStatus,
	pub url: String,
}

pub struct FetchMangaInfo {
	pub dir: String,
	pub branches: Vec<String>,
}

pub struct ChapterContainer {
	pub index: i64,
	pub item: Chapter,
}
