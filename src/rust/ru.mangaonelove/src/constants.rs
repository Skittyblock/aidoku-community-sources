use const_format::formatcp;

pub const BASE_URL: &str = "https://mangaonelove.site";
pub const MANGA_DIR: &str = "manga";
pub const MANGA_BASE_URL: &str = formatcp!("{}/{}", BASE_URL, MANGA_DIR);
pub const PAGE_DIR: &str = "page";
pub const SEARCH_OFFSET_STEP: i32 = 10;

pub const BASE_URL_READMANGA: &str = "https://readmanga.live";
