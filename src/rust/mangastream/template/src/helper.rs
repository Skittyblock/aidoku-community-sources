use aidoku::{
    std::{String, StringRef, Vec, html::Node}, MangaStatus, Listing, prelude::format,
};

// generate url for listing page
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

// return the manga status 
pub fn manga_status(status: String) -> MangaStatus {
    if status == "ONGOING" || status == "DEVAM EDIYOR" {
        return MangaStatus::Ongoing;
    } else if status == "COMPLETED" || status == "TAMAMLANDI" {
        return MangaStatus::Completed;
    } else if status == "HIATUS" || status == "DURDURULDU" {
        return MangaStatus::Hiatus;
    } else if status == "CANCELLED" || status == "BIRAKILDI" || status == "DROPPED" {
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
    if !query.is_empty() {
        url.push_str("/page/");
        url.push_str(&i32_to_string(page));
        url.push_str("?s=");
        url.push_str(&query);
        return url;
    } else {
        url.push_str("/?page=");
        url.push_str(&i32_to_string(page));
    }
    if !included_tags.is_empty() {
        for tag in included_tags {
            url.push_str("&genre%5B%5D=");
            url.push_str(&tag);
        }
    }
    if !status.is_empty() {
        url.push_str("&status=");
        url.push_str(&status);
    }
    if !manga_type.is_empty() {
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

//get the image sources as some images are in base64 format
pub fn get_image_src(node:Node) -> String{
    let image = node.select("img").first().attr("src").read();
    if image.starts_with("data") || image == "" {
        return node.select("img").first().attr("data-lazy-src").read();
    }else {
        return image;
    }
}
