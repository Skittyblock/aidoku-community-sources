use aidoku::{
	error::Result, prelude::*, std::net::HttpMethod, std::net::Request, std::ArrayRef, std::String,
	std::ValueRef, std::Vec,
};

use prost::bytes::Bytes;
use prost::Message;

use base64ct::{Base64, Encoding};
use rsa::{pkcs8::DecodePrivateKey, RsaPrivateKey, RsaPublicKey};

pub mod protobuf {
	include!(concat!(env!("OUT_DIR"), "/dmzj.comic.rs"));
}

// 地区
pub fn type_list() -> [i32; 7] {
	[0, 2304, 2305, 2306, 2307, 2308, 8453]
}

// 读者
pub fn reader_list() -> [i32; 4] {
	[0, 3262, 3263, 3264]
}

// 连载状态
pub fn status_list() -> [i32; 3] {
	[0, 2309, 2310]
}

// 分类
pub fn genre_list() -> [i32; 42] {
	[
		0, 4, 3243, 3242, 17, 3244, 3245, 3249, 3248, 3246, 16, 14, 7, 6, 5, 8, 9, 13, 12, 11, 10,
		3250, 3251, 5806, 5345, 5077, 5848, 6316, 7900, 7568, 6437, 4518, 4459, 3254, 3253, 3252,
		3255, 6219, 3328, 3365, 3326, 3325,
	]
}

pub fn encodeURI(string: &String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if (b'a' <= curr && curr <= b'z')
			|| (b'A' <= curr && curr <= b'Z')
			|| (b'0' <= curr && curr <= b'9')
			|| (curr == b';'
				|| curr == b',' || curr == b'/'
				|| curr == b'?' || curr == b':'
				|| curr == b'@' || curr == b'&'
				|| curr == b'=' || curr == b'+'
				|| curr == b'$')
			|| (curr == b'-'
				|| curr == b'_' || curr == b'.'
				|| curr == b'!' || curr == b'~'
				|| curr == b'*' || curr == b'\''
				|| curr == b'(' || curr == b')')
			|| (curr == b'#')
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

pub fn GET(url: String) -> Request {
	Request::new(&url, HttpMethod::Get)
    .header("Referer", "https://www.dmzj.com/")
    .header("User-Agent",
    "Mozilla/5.0 (Linux; Android 10) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.93 Mobile Safari/537.36 Aidoku/1.0")
}

const KEY:&str = "MIICeAIBADANBgkqhkiG9w0BAQEFAASCAmIwggJeAgEAAoGBAK8nNR1lTnIfIes6oRWJNj3mB6OssDGx0uGMpgpbVCpf6+VwnuI2stmhZNoQcM417Iz7WqlPzbUmu9R4dEKmLGEEqOhOdVaeh9Xk2IPPjqIu5TbkLZRxkY3dJM1htbz57d/roesJLkZXqssfG5EJauNc+RcABTfLb4IiFjSMlTsnAgMBAAECgYEAiz/pi2hKOJKlvcTL4jpHJGjn8+lL3wZX+LeAHkXDoTjHa47g0knYYQteCbv+YwMeAGupBWiLy5RyyhXFoGNKbbnvftMYK56hH+iqxjtDLnjSDKWnhcB7089sNKaEM9Ilil6uxWMrMMBH9v2PLdYsqMBHqPutKu/SigeGPeiB7VECQQDizVlNv67go99QAIv2n/ga4e0wLizVuaNBXE88AdOnaZ0LOTeniVEqvPtgUk63zbjl0P/pzQzyjitwe6HoCAIpAkEAxbOtnCm1uKEp5HsNaXEJTwE7WQf7PrLD4+BpGtNKkgja6f6F4ld4QZ2TQ6qvsCizSGJrjOpNdjVGJ7bgYMcczwJBALvJWPLmDi7ToFfGTB0EsNHZVKE66kZ/8Stx+ezueke4S556XplqOflQBjbnj2PigwBN/0afT+QZUOBOjWzoDJkCQClzo+oDQMvGVs9GEajS/32mJ3hiWQZrWvEzgzYRqSf3XVcEe7PaXSd8z3y3lACeeACsShqQoc8wGlaHXIJOHTcCQQCZw5127ZGs8ZDTSrogrH73Kw/HvX55wGAeirKYcv28eauveCG7iyFR0PFB/P/EDZnyb+ifvyEFlucPUI0+Y87F";

pub fn DECODE(base64: &String) -> protobuf::ComicDetailResponse {
	let keyByte = Base64::decode_vec(KEY).unwrap();
	let privateKey = rsa::RsaPrivateKey::from_pkcs8_der(&keyByte).unwrap();
	let r = Base64::decode_vec(&base64).unwrap();
	const BLOCK_SIZE: usize = 128;
	let mut iter = r.chunks(BLOCK_SIZE);

	let mut rr = Vec::new();
	while let Some(ptr) = iter.next() {
		rr = [
			rr,
			privateKey
				.decrypt(rsa::PaddingScheme::PKCS1v15Encrypt, ptr)
				.unwrap(),
		]
		.concat();
	}

	return prost::Message::decode(Bytes::from(rr)).unwrap();
}
