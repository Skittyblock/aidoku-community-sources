use const_format::formatcp;

pub const BASE_URL: &str = "https://t.readmanga.io";
pub const BASE_SEARCH_URL: &str = formatcp!("{}/{}", BASE_URL, "search/advancedResults?");

pub const SEARCH_OFFSET_STEP: i32 = 50;
