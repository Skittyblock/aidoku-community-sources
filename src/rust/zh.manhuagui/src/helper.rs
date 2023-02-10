use alloc::{string::String, vec::Vec};

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
