use aidoku::{
    prelude::*,
	std::String, std::ArrayRef, std::Vec, MangaStatus,
    std::html::Node,
};

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

pub fn get_image_url(obj: Node) -> String {
    let mut img;
    img = obj.attr("data-src").read();
    if img.len() == 0 {
        img = obj.attr("data-lazy-src").read();
    }
    if img.len() == 0 {
        img = obj.attr("src").read();
    }
    if img.len() == 0 {
        img = obj.attr("srcset").read();

    }
    // img = img.replace("-175x238", "").replace("-350x476", "").replace("-110x150", "");
    img = String::from(img.trim());
    println!("image `{}`", img);
    return img;
}
