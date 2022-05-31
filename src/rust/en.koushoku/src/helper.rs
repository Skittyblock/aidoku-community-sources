use aidoku::{prelude::*, std::String, std::Vec};

pub fn urlencode(string: String) -> String {
    let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
    let hex = "0123456789abcdef".as_bytes();
    let bytes = string.as_bytes();

    for byte in bytes {
        let curr = *byte;
        if (b'a' <= curr && curr <= b'z')
            || (b'A' <= curr && curr <= b'Z')
            || (b'0' <= curr && curr <= b'9')
        {
            result.push(curr);
        } else {
            result.push(b'%');
            result.push(hex[curr as usize >> 4]);
            result.push(hex[curr as usize & 15]);
        }
    }

    String::from_utf8(result).unwrap_or(String::new())
}

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

pub fn get_cover_url(id: String) -> String {
    return format!("https://cdn.koushoku.org/data/{}/1/512.png", id);
}

pub fn get_manga_id(link: String) -> String {
    // https://koushoku.org/archive/8918
    let str = link.split("/").nth(4).unwrap_or("");  
    return String::from(str);
}

pub fn get_manga_id_from_path(path: String) -> String {
    // /archive/8918
    let str = path.split("/").nth(2).unwrap_or("");
    return String::from(str);
}

pub fn build_search_url(query: String, sort_type: String, included_tags: Vec<String>, excluded_tags: Vec<String>, ascending: bool, page: i32) -> String {
    
    let mut url = String::new();

    if query.len() > 0 || included_tags.len() > 0 || excluded_tags.len() > 0 {
        // search page
        url.push_str(format!("https://koushoku.org/search").as_str());
        url.push_str("?");
    } else {
        url.push_str(format!("https://koushoku.org/").as_str());
        url.push_str("?");
    }
    url.push_str(format!("page={}&sort={}&order={}&q={}",
    i32_to_string(page),
    sort_type,
    if ascending { "asc" } else { "desc" },
    urlencode(query).as_str()
    ).as_str());
    let mut query_params = String::new();
    if !included_tags.is_empty() {
        query_params.push_str("tag&:");
        query_params.push_str(&included_tags.join(","));
    }
    if !excluded_tags.is_empty() {
        query_params.push_str("-tag:");
        query_params.push_str(&excluded_tags.join(","));
    }
    
    url.push_str(urlencode(query_params).as_str());

    url
}

pub fn get_page(url: String) -> i32 {
    let mut params = url.split("&");
    let mut page = 1;
    while let Some(param) = params.next() {
        if param.starts_with("page=") {
            page = param[5..].parse::<i32>().unwrap_or(1);
            break;
        }
    }
    return page;
}
