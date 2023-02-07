use alloc::{string::String, vec::Vec};

pub fn encode_uri(string: &String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if (b'a'..=b'z').contains(&curr)
			|| (b'A'..=b'Z').contains(&curr)
			|| (b'0'..=b'9').contains(&curr)
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
