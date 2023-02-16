use aidoku::{
	error::Result,
	helpers::uri::encode_uri_component,
	prelude::*,
	std::String,
	std::{
		net::{HttpMethod, Request},
		StringRef, Vec,
	},
};
use md5::{Digest, Md5};

const HEX_CHARS: [char; 16] = [
	'0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

pub fn md5(str: &String) -> String {
	let mut hasher = Md5::default();
	hasher.update(str);

	let bytes = hasher.finalize();
	let mut result = String::new();

	for b in bytes {
		let x = ((b >> 4) & 0xf) as usize;
		let y = (b & 0xf) as usize;
		result.push(HEX_CHARS[x]);
		result.push(HEX_CHARS[y]);
	}

	result
}

const GSN_KEY: &str = "4e0a48e1c0b54041bce9c8f0e036124d";

pub fn generate_gsn_hash(args: &mut Vec<(String, String)>) -> String {
	let mut temp = String::new();

	args.sort_by(|a, b| a.0.cmp(&b.0));

	temp.push_str(GSN_KEY);
	temp.push_str("GET");

	for a in args {
		temp.push_str(&a.0);
		temp.push_str(&encode_uri_component(String::from(&a.1)));
	}

	temp.push_str(GSN_KEY);

	md5(&temp)
}

pub fn generate_get_query(args: &mut Vec<(String, String)>) -> String {
	args.push((String::from("gak"), String::from("android_manhuaren2")));

	let gsn = generate_gsn_hash(args);

	args.push((String::from("gsn"), String::from(&gsn)));

	let mut qs = String::new();

	for (i, a) in args.iter().enumerate() {
		if i > 0 {
			qs.push('&');
		}

		let v = encode_uri_component(String::from(&a.1));
		qs.push_str(&format!("{}={}", a.0, v));
	}

	qs
}

pub fn request<T: AsRef<str>>(url: T, method: HttpMethod) -> Result<String> {
	Request::new(url, method)
		.header("X-Yq-Yqci", "{\"le\": \"zh\"}")
		.string()
}

pub fn stringref_unwrap_or_fallback(val: Result<StringRef>, fallback: String) -> String {
	match val {
		Ok(val) => {
			let str = val.read();
			if str.is_empty() {
				fallback
			} else {
				str
			}
		}
		Err(_) => fallback,
	}
}

pub fn stringref_unwrap_or_empty(val: Result<StringRef>) -> String {
	stringref_unwrap_or_fallback(val, String::new())
}
