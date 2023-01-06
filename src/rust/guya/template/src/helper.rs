use aidoku::{
	std::String, std::ArrayRef, std::Vec, MangaStatus, prelude::format,
};

pub fn extract_f32_from_string(title: String, text: String) -> f32 {  
    text.replace(&title, "")
        .chars()
        .filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.')
        .collect::<String>()
        .split(" ")
        .collect::<Vec<&str>>().into_iter()
        .map(|a| a.parse::<f32>().unwrap_or(0.0))
        .find(|a| *a > 0.0)
        .unwrap_or(0.0)
}

pub fn append_protocol(url: String) -> String {
    if !url.starts_with("http") {
        return format!("{}{}", "https:", url);
    } else {
        return url;
    }
}

pub fn https_upgrade(url: String) -> String {
    return url.replacen("http://", "https://", 1);
}

pub fn urlencode(string: String) -> String {
    let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
    let hex = "0123456789abcdef".as_bytes();
    let bytes = string.as_bytes();
    
    for byte in bytes {
        let curr = *byte;
        if (b'a' <= curr && curr <= b'z')
            || (b'A' <= curr && curr <= b'Z')
            || (b'0' <= curr && curr <= b'9') {
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

pub fn join_string_array(array: ArrayRef, delimeter: String) -> String {
    let mut string = String::new();
    let mut at = 0;
    for item in array {
        if at != 0 {
            string.push_str(&delimeter);
        }
        string.push_str(item.as_node().text().read().as_str());
        at += 1;
    }
    return string;
}

pub fn status_from_string(status: String) -> MangaStatus {
    if status == "Ongoing" {
        return MangaStatus::Ongoing;
    } else if status == "Completed" {
        return MangaStatus::Completed;
    } else if status == "Hiatus" {
        return MangaStatus::Hiatus;
    } else if status == "Cancelled" {
        return MangaStatus::Cancelled;
    } else {
        return MangaStatus::Unknown;
    }
}

pub fn is_numeric_char(c: char) -> bool {
    return (c >= '0' && c <= '9') || c == '.';
}

pub fn get_chapter_number(id: String) -> f32 {
    let mut number_string = String::new();
    let mut i = id.len() - 1;
    for c in id.chars().rev() {
        if !is_numeric_char(c) {
            number_string = String::from(&id[i + 1..]);
            break;
        }
        i -= 1;
    }
    if number_string.len() == 0 {
        return 0.0;
    }
    return number_string.parse::<f32>().unwrap_or(0.0);
}

pub fn string_replace(string: String, search: String, replace: String) -> String {
    let mut result = String::new();
    let mut at = 0;
    for c in string.chars() {
        if c == search.chars().next().unwrap() {
            if string[at..].starts_with(&search) {
                result.push_str(&replace);
                at += search.len();
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
        at += 1;
    }
    return result;
}

pub fn stupidencode(string: String) -> String {
    let mut result = String::new();
    for c in string.chars() {
        if c.is_alphanumeric() {
            result.push(c);
        } else if c == ' ' {
            result.push('_');
        }
    }
    return result;
}
