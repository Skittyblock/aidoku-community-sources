use const_format::formatcp;

pub const BASE_URL: &str = "https://desu.city";
pub const API_URL: &str = "manga/api";
pub const BASE_API_URL: &str = formatcp!("{BASE_URL}/{API_URL}");
pub const RATE_LIMIT: i32 = 3;
pub const RATE_LIMIT_PERIOD: i32 = 1;
pub const USER_AGENT: &str = "Aidoku";
pub const PAGE_LIMIT: i32 = 20;
