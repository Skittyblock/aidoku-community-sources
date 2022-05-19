use aidoku::{
    prelude::format,
    std::{String, StringRef, Vec},
    MangaStatus,
};
use wpcomics_template::helper::urlencode;

pub fn listing_mapping(listing: String) -> String {
    String::from(match listing.as_str() {
        "Hot" => "hot-comic",
        "Popular" => "popular-comic",
        _ => "",
    })
}

pub fn status_map(arg1: String) -> MangaStatus {
    return match arg1.as_str() {
        "Ongoing" => MangaStatus::Ongoing,
        "Completed" => MangaStatus::Completed,
        _ => MangaStatus::Unknown,
    };
}

pub fn convert_time(time_ago: String) -> f64 {
    #[allow(unused_assignments)]
    let time_object = StringRef::from(time_ago).0;
    return time_object
        .as_date("MM/dd/yy", Some("en_US"), None)
        .unwrap_or(0.0);
}

pub fn trunc_trailing_comic(title: String) -> String {
    return title
        .chars()
        .rev()
        .collect::<String>()
        .replacen("cimoC", "", 1)
        .chars()
        .rev()
        .collect::<String>();
}

pub fn get_search_url(
    base_url: String,
    query: String,
    include: Vec<String>,
    exclude: Vec<String>,
    completed: String,
    page: i32,
) -> String {
    if query.len() == 0 && completed.len() == 0 && include.len() == 0 && exclude.len() == 0 {
        return String::from("https://comiconlinefree.net")
    } else {
        format!(
            "{base_url}/advanced-search?key={query}&wg={}&wog={}&status={completed}&page={page}",
            urlencode(include.join(",")),
            urlencode(exclude.join(",")),
        )
    }
}
