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

// get parameter page from url as i32
pub fn get_page(url: String) -> i32 {
    let mut page = 1;
    let mut index = url.find("page=").unwrap_or(0);
    if index == 0 {
        return page;
    }
    index += 5;
    let end = url.find('&').unwrap_or(url.len());
    if end == index {
        return page;
    }
    page = url[index..end].parse().unwrap_or(page);
    return page;
}
