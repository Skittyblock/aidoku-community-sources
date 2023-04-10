use aidoku::std::Vec;
use serde::Deserialize;

extern crate alloc;
use alloc::borrow::Cow;

#[derive(Deserialize, Debug, Clone)]
pub enum Nepnep<'a> {
	Directory { items: Vec<Directory<'a>> },
	HotUpdate { items: Vec<HotUpdate<'a>> },
}

pub trait Pattern {
	fn start(&self) -> &str;
	fn end(&self) -> &str;
	fn path(&self) -> &str;
}

impl Pattern for Nepnep<'_> {
	fn start(&self) -> &str {
		match self {
			Nepnep::Directory { .. } => "vm.Directory =",
			Nepnep::HotUpdate { .. } => "vm.HotUpdateJSON =",
		}
	}

	fn end(&self) -> &str {
		"];"
	}

	fn path(&self) -> &str {
		match self {
			Nepnep::Directory { .. } => "/search/",
			Nepnep::HotUpdate { .. } => "/hot.php",
		}
	}
}

pub trait Size {
	fn len(&self) -> usize;
}

impl Size for Nepnep<'_> {
	fn len(&self) -> usize {
		match self {
			Nepnep::Directory { items } => items.len(),
			Nepnep::HotUpdate { items } => items.len(),
		}
	}
}

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Directory<'a> {
	#[serde(rename = "i")]
	pub id: Cow<'a, str>,

	#[serde(rename = "s")]
	pub title: Cow<'a, str>,

	// time in epoch
	#[serde(rename = "lt")]
	pub last_updated: i32,

	#[serde(rename = "y")]
	pub year: Cow<'a, str>,

	#[serde(rename = "v")]
	pub views: Cow<'a, str>,

	#[serde(rename = "vm")]
	pub views_month: Cow<'a, str>,

	#[serde(rename = "al")]
	pub alt_titles: Vec<Cow<'a, str>>,
}

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct HotUpdate<'a> {
	#[serde(rename = "IndexName")]
	pub id: Cow<'a, str>,

	#[serde(rename = "SeriesName")]
	pub title: Cow<'a, str>,
}

pub enum SortOptions {
	AZ,
	ZA,
	RecentlyReleasedChapter,
	YearReleasedNewest,
	YearReleasedOldest,
	MostPopularAllTime,
	MostPopularMonthly,
	LeastPopular,
}

impl From<i32> for SortOptions {
	fn from(value: i32) -> Self {
		match value {
			0 => SortOptions::AZ,
			1 => SortOptions::ZA,
			2 => SortOptions::RecentlyReleasedChapter,
			3 => SortOptions::YearReleasedNewest,
			4 => SortOptions::YearReleasedOldest,
			5 => SortOptions::MostPopularAllTime,
			6 => SortOptions::MostPopularMonthly,
			7 => SortOptions::LeastPopular,
			_ => SortOptions::AZ,
		}
	}
}
