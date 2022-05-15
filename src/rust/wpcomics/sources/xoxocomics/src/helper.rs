use aidoku::{
    std::{String, StringRef}, MangaStatus
};
use wpcomics_template::helper::i32_to_string;

// MARK: Mappings
pub fn get_tag_id(genre: i64) -> String {
    return String::from(match genre {
        0 => "marvel",
        1 => "dc-comics",
        2 => "action",
        3 => "adventure",
        4 => "anthology",
        5 => "anthropomorphic",
        6 => "biography",
        7 => "children",
        8 => "comedy",
        9 => "crime",
        10 => "cyborgs",
        11 => "dark-horse",
        12 => "demons",
        13 => "drama",
        14 => "fantasy",
        15 => "family",
        16 => "fighting",
        17 => "gore",
        18 => "graphic-novels",
        19 => "historical",
        20 => "horror",
        21 => "leading-ladies",
        22 => "literature",
        23 => "magic",
        24 => "manga",
        25 => "martial-arts",
        26 => "mature",
        27 => "mecha",
        28 => "military",
        29 => "movie-cinematic-link",
        30 => "mystery",
        31 => "mythology",
        32 => "psychological",
        33 => "personal",
        34 => "political",
        35 => "post-apocalyptic",
        36 => "pulp",
        37 => "robots",
        38 => "romance",
        39 => "sci-fi",
        40 => "slice-of-life",
        41 => "science-fiction",
        42 => "sport",
        43 => "spy",
        44 => "superhero",
        45 => "supernatural-",
        46 => "suspense",
        47 => "thriller",
        48 => "vampires",
        49 => "vertigo",
        50 => "video-games",
        51 => "war",
        52 => "western",
        53 => "zombies",
        _ => ""
    });
}

pub fn listing_map(listing: String) -> String {
    let url: &str = match listing.as_str() {
        "Popular" => "popular-comics",
        "Hot" => "hot",
        "Completed" => "status/completed",
        "Ongoing" => "status/ongoing",
        _ => "",
    };
    return String::from(url);
}

pub fn status_map (arg1: String) -> MangaStatus {
    return match arg1.as_str() {
        "Ongoing" => MangaStatus::Ongoing,
        "Completed" => MangaStatus::Completed,
        _ => MangaStatus::Unknown,
    };
}

// MARK: Other utilities
pub fn get_search_url(base_url: String, query: String, genre: String, page: i32) -> String {
    let mut url = String::new();
    url.push_str(&base_url);
    if query.len() > 0 {
        url.push_str("/search?page=");
        url.push_str(i32_to_string(page).as_str());
        url.push_str("&keyword=");
        url.push_str(&query);
    } else if genre.len() > 0 {
        url.push_str("/genre/");
        url.push_str(&genre);
        url.push_str("?page=");
        url.push_str(i32_to_string(page).as_str());
    }
    return url;
}

pub fn convert_time(time_ago: String) -> f64 {
    #[allow(unused_assignments)]
    let time_object = StringRef::from(time_ago).0;
    return time_object.as_date("MM/dd/yyyy", Some("en_US"), None).unwrap_or(0.0);
}
