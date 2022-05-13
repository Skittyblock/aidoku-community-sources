use aidoku::{std::{String, StringRef, html::Node}, std::Vec, MangaStatus, prelude::format, Listing};

//generate url for listing page
pub fn get_listing_url(base_url: String, pathname: String, listing: Listing, page: i32) -> String {
    let list_type = match listing.name.as_str() {
        "Latest" => "order=update",
        "Popular" => "order=popular",
        "New" => "order=latest",
        _ => "",
    };
    if page == 1 {
        return format!("{}/{}/?{}", base_url, pathname,list_type)
    }
    else {
        return format!("{}/{}/?page={}&{}", base_url, pathname, page,list_type)
    }
}

// return the tag id that will be used to filter the genre
pub fn get_tag_id(base_url: String, tag: String) -> String {
    let tag_id;
    if base_url.contains("alpha-scans") {
        let id =  match tag.as_str() {
            "Action" => "14",
            "Adventure" => "16",
            "Comedy" => "3",
            "Drama" => "10",
            "Ecchi" => "70",
            "Fantasy" => "11",
            "Harem" => "47",
            "Historical" => "78",
            "Horror" => "90",
            "Isekai" => "56",
            "Josei" => "79",
            "Martial Arts" => "32",
            "Medical" => "67",
            "Mystery" => "20",
            "Psychological" => "64",
            "Romance" => "18",
            "School Life" => "4",
            "Sci-Fi" => "66",
            "Seinen" => "33",
            "Shoujo" => "12",
            "Shounen" => "7",
            "Slice of Life" => "8",
            "Supernatural" => "5",
            "System" => "84",
            _ => "",
        };
        tag_id = String::from(id);
    }
    else if base_url.contains("readkomik") {
        let id =  match tag.as_str() {
            "Action" => "2",
            "Acttion" => "949",
            "Adult" => "523",
            "Adventure" => "18",
            "Battle Royale" => "1376",
            "Comedy" => "12",
            "Crime" => "727",
            "Demon" => "1467",
            "Demons" => "473",
            "Drama" => "137",
            "Ecchi" => "141",
            "Fansaty" => "829",
            "Fantasi" => "1328",
            "Fantasy" => "3",
            "Fantasy Shounen" => "1611",
            "Full Color" => "1370",
            "Game" => "15",
            "Gender Bender" => "544",
            "Gore" => "1509",
            "Harem" => "139",
            "Historical" => "344",
            "Horror" => "274",
            "Hot blood" => "1371",
            "Isekai" => "13",
            "Josei" => "561",
            "Lolicon" => "764",
            "Magic" => "4",
            "Manga" => "1212",
            "Manhua" => "1243",
            "Manhwa" => "1064",
            "Martial Arts" => "22",
            "Mature" => "138",
            "Mecha" => "507",
            "Medical" => "797",
            "Murim" => "1547",
            "Mystery" => "275",
            "Otherworld" => "1627",
            "Post-Apocalyptic" => "1244",
            "Psychological" => "592",
            "Rebirth" => "839",
            "Reincarnation" => "349",
            "Revenge" => "1339",
            "Romance" => "308",
            "School Life" => "160",
            "Sci-Fi" => "23",
            "Seinen" => "140",
            "Shotacon" => "480",
            "Shoujo" => "516",
            "Shoujo Ai" => "689",
            "Shounen" => "161",
            "Slice of Life" => "162",
            "Smut" => "765",
            "Sports" => "163",
            "Supernatural" => "165",
            "Survival" => "1245",
            "System" => "1282",
            "Thriller" => "922",
            "Time Travel" => "1018",
            "Tragedy" => "276",
            "Yuri" => "720",
            "Zombies" => "1260",
            _ => "",
        };
        tag_id = String::from(id);
    }
    else {
        tag_id =  String::from(tag.to_lowercase().replace(" ", "-").as_str());
    }
    return tag_id;
}

// return the manga status 
pub fn manga_status(status: String) -> MangaStatus {
    if status == "ONGOING" {
        return MangaStatus::Ongoing;
    } else if status == "COMPLETED" {
        return MangaStatus::Completed;
    } else if status == "HIATUS" {
        return MangaStatus::Hiatus;
    } else if status == "CANCELLED" {
        return MangaStatus::Cancelled;
    } else {
        return MangaStatus::Unknown;
    }
}

//converts integer(i32) to string
pub fn i32_to_string(mut integer: i32) -> String {
    if integer == 0 {
        return String::from("0");
    }
    let mut string = String::with_capacity(11);
    let pos = if integer < 0 {
        string.insert(0, '-');
        1
    } else {
        0
    };
    while integer != 0 {
        let mut digit = integer % 10;
        if pos == 1 {
            digit *= -1;
        }
        string.insert(pos, char::from_u32((digit as u32) + ('0' as u32)).unwrap());
        integer /= 10;
    }
    return string;
}

// return chpater number from string
pub fn get_chapter_number(id: String) -> f32 {
    let values: Vec<&str> = id.split(" ").collect();
    return values[1].parse::<f32>().unwrap_or(0.0);
}

// generates the search, filter and homepage url 
pub fn get_search_url(base_url: String, query: String,page: i32, included_tags: Vec<String>, status: String, manga_type: String, traverse_pathname: String) -> String {
    let mut url = String::new();
    url.push_str(&base_url);
    url.push_str("/");
    url.push_str(&traverse_pathname);
    if query.is_empty() && included_tags.is_empty() && status.is_empty() && manga_type.is_empty() {
        match page {
            1 => return url,
            _ => {
                url.push_str("?page=");
                url.push_str(&i32_to_string(page));
                return url;
            },
        }            
    }
    if query.len() > 0 {
        url.push_str("/page/");
        url.push_str(&i32_to_string(page));
        url.push_str("?s=");
        url.push_str(&query);
        return url;
    } else {
        url.push_str("/?page=");
        url.push_str(&i32_to_string(page));
        
    }
    if included_tags.len() > 0 {
        for tag in included_tags {
            url.push_str("&genre%5B%5D=");
            url.push_str(&tag);
        }
    }
    if status.len() > 0 {
        url.push_str("&status=");
        url.push_str(&status);
    }
    if manga_type.len() > 0 {
        url.push_str("&type=");
        url.push_str(&manga_type);
    }
    return url;
}

// return the date depending on the language
pub fn get_date(id: String, date_format: String, locale: &str, raw_date: StringRef) -> f64{
    match id.contains("asurascanstr"){
        true => return raw_date.0.as_date("MMMM d, yyyy", Some("tr_TR"), None).unwrap_or(0.0),
        _ => return raw_date.0.as_date(date_format.as_str(), Some(locale), None).unwrap_or(0.0),
    }
}

pub fn get_image_src(node:Node) -> String{
    let image = node.select("img").first().attr("src").read();
    if image.starts_with("data") || image == "" {
        return node.select("img").first().attr("data-lazy-src").read();
    }else {
        return image;
    }
}