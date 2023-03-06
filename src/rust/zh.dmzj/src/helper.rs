use aidoku::{std::net::HttpMethod, std::net::Request, std::String, std::Vec};

use prost::bytes::Bytes;

use base64ct::{Base64, Encoding};
use rsa::pkcs8::DecodePrivateKey;

pub mod protobuf {
	include!(concat!(env!("OUT_DIR"), "/dmzj.chapter_images.rs"));
	include!(concat!(env!("OUT_DIR"), "/dmzj.comic_detail.rs"));
}

pub fn encode_uri(string: &String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if curr.is_ascii_lowercase()
			|| curr.is_ascii_uppercase()
			|| curr.is_ascii_digit()
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

	String::from_utf8(result).unwrap_or_default()
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
	string
}

pub fn get(url: &str) -> Request {
	Request::new(url, HttpMethod::Get)
    .header("Referer", "https://www.dmzj.com/")
    .header("User-Agent",
    "Mozilla/5.0 (Linux; Android 10) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.93 Mobile Safari/537.36 Aidoku/1.0")
}

const KEY:&str = "MIICeAIBADANBgkqhkiG9w0BAQEFAASCAmIwggJeAgEAAoGBAK8nNR1lTnIfIes6oRWJNj3mB6OssDGx0uGMpgpbVCpf6+VwnuI2stmhZNoQcM417Iz7WqlPzbUmu9R4dEKmLGEEqOhOdVaeh9Xk2IPPjqIu5TbkLZRxkY3dJM1htbz57d/roesJLkZXqssfG5EJauNc+RcABTfLb4IiFjSMlTsnAgMBAAECgYEAiz/pi2hKOJKlvcTL4jpHJGjn8+lL3wZX+LeAHkXDoTjHa47g0knYYQteCbv+YwMeAGupBWiLy5RyyhXFoGNKbbnvftMYK56hH+iqxjtDLnjSDKWnhcB7089sNKaEM9Ilil6uxWMrMMBH9v2PLdYsqMBHqPutKu/SigeGPeiB7VECQQDizVlNv67go99QAIv2n/ga4e0wLizVuaNBXE88AdOnaZ0LOTeniVEqvPtgUk63zbjl0P/pzQzyjitwe6HoCAIpAkEAxbOtnCm1uKEp5HsNaXEJTwE7WQf7PrLD4+BpGtNKkgja6f6F4ld4QZ2TQ6qvsCizSGJrjOpNdjVGJ7bgYMcczwJBALvJWPLmDi7ToFfGTB0EsNHZVKE66kZ/8Stx+ezueke4S556XplqOflQBjbnj2PigwBN/0afT+QZUOBOjWzoDJkCQClzo+oDQMvGVs9GEajS/32mJ3hiWQZrWvEzgzYRqSf3XVcEe7PaXSd8z3y3lACeeACsShqQoc8wGlaHXIJOHTcCQQCZw5127ZGs8ZDTSrogrH73Kw/HvX55wGAeirKYcv28eauveCG7iyFR0PFB/P/EDZnyb+ifvyEFlucPUI0+Y87F";

fn decode(base64: &str) -> Vec<u8> {
	let key_byte = Base64::decode_vec(KEY).unwrap();
	let private_key = rsa::RsaPrivateKey::from_pkcs8_der(&key_byte).unwrap();
	let r = Base64::decode_vec(base64).unwrap();
	const BLOCK_SIZE: usize = 128;
	let iter = r.chunks(BLOCK_SIZE);

	let mut rr = Vec::new();
	for ptr in iter {
		rr = [
			rr,
			private_key
				.decrypt(rsa::PaddingScheme::PKCS1v15Encrypt, ptr)
				.unwrap(),
		]
		.concat();
	}
	rr
}

pub fn decode_as_comic_detail(
	base64: &str,
) -> Result<protobuf::ComicDetailResponse, prost::DecodeError> {
	prost::Message::decode(Bytes::from(decode(base64)))
}

/*
pub fn decode_as_chapter_images(base64: &str) -> Result<protobuf::ResponseDto, prost::DecodeError> {
	prost::Message::decode(Bytes::from(decode(base64)))
}
*/
