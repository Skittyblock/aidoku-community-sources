use aidoku::std::{
    Vec, String
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub enum Nepnep {
    Directory { items: Vec<Directory> },
    HotUpdate { items: Vec<HotUpdate> }
}

pub trait Pattern {
    fn start(&self) -> &str;
    fn end(&self) -> &str;
    fn path(&self) -> &str;
}

impl Pattern for Nepnep {
    fn start(&self) -> &str {
        match self {
            Nepnep::Directory { .. } => "vm.Directory =",
            Nepnep::HotUpdate { .. } => "vm.HotUpdateJSON ="
        }
    }

    fn end(&self) -> &str {
        "];"
    }

    fn path(&self) -> &str {
        match self {
            Nepnep::Directory { .. } => "/search/",
            Nepnep::HotUpdate { .. } => "/hot.php"
        }
    }
}

pub trait Size {
    fn len(&self) -> usize;
}

impl Size for Nepnep {
    fn len(&self) -> usize {
        match self {
            Nepnep::Directory { items } => items.len(),
            Nepnep::HotUpdate { items } => items.len(),
        }
    }
}


#[derive(Default, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Directory {
	#[serde(rename = "i")]
	pub id: String,

	#[serde(rename = "s")]
	pub title: String,

    // time in epoch
	#[serde(rename = "lt")]
    pub last_updated: i32,

	#[serde(rename = "y")]
    pub year: String,

	#[serde(rename = "v")]
    pub views: String,

	#[serde(rename = "vm")]
    pub views_month: String,

	#[serde(rename = "al")]
	pub alt_titles: Vec<String>,
}

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct HotUpdate {
	#[serde(rename = "IndexName")]
	pub id: String,

	#[serde(rename = "SeriesName")]
	pub title: String,
}
