use aidoku::{
	error::{AidokuError, Result},
	prelude::*,
	std::html::Node,
	std::String,
	std::Vec,
};

pub fn extract_f32_from_string(title: String, text: String) -> f32 {
	text.replace(&title, "")
		.replace(|a: char| a == ',' || a == '_' || a == '-', ".")
		.chars()
		.filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.')
		.collect::<String>()
		.split(' ')
		.collect::<Vec<&str>>()
		.into_iter()
		.map(|a| a.parse::<f32>().unwrap_or(0.0))
		.find(|a| *a > 0.0)
		.unwrap_or(-1.0)
}

pub fn append_protocol(url: String) -> String {
	if !url.starts_with("http") {
		return format!("{}{}", "https:", url);
	} else {
		url
	}
}

pub fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789ABCDEF".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if curr.is_ascii_alphanumeric() {
			result.push(curr);
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}
	String::from_utf8(result).unwrap_or_default()
}

fn parse_email_protected<T: AsRef<str>>(data: T) -> Result<String> {
	let data = data.as_ref();
	if let Ok(key) = u32::from_str_radix(&data[0..2], 16) {
		let mut email = String::with_capacity(data.len() / 2 - 1);
		let mut n = 2;

		while n < data.len() {
			if let Ok(chrcode) = u32::from_str_radix(&data[n..n + 2], 16)
			   && let Some(chr) = char::from_u32(chrcode ^ key) {
				email.push(chr);
			}
			n += 2;
		}
		Ok(email)
	} else {
		Err(AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		})
	}
}

pub fn email_unprotected(html: &Node) {
	let elems = html.select(".__cf_email__");
	for elem in elems.array() {
		if let Ok(mut node) = elem.as_node()
		   && let Ok(email) = parse_email_protected(node.attr("data-cfemail").read()) {
			node.set_text(email).ok();
		}
	}
}
