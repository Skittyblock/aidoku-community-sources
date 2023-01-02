use aidoku::{error::Result, std::String, std::ValueRef, std::Vec};
use alloc::string::ToString;

pub fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if curr.is_ascii_lowercase() || curr.is_ascii_uppercase() || curr.is_ascii_digit() {
			result.push(curr);
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}

	String::from_utf8(result).unwrap_or_default()
}

pub fn get_id(value: ValueRef) -> Result<String> {
	let id = value.as_int().unwrap_or(0) as i32;
	Ok(if id != 0 {
		id.to_string()
	} else {
		value.as_string()?.read()
	})
}
