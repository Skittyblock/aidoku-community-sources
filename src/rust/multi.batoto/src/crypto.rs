extern crate alloc;
use crate::evpkdf::evpkdf;
use aes::{
	cipher::{BlockDecryptMut, KeyIvInit},
	Aes256,
};

use alloc::{format, string::String, vec, vec::Vec};
use block_padding::Pkcs7;
use cbc::Decryptor;
use md5::Md5;

type Aes256CbcDec = Decryptor<Aes256>;

const AES_128_LENGTH: usize = 16;
const AES_192_LENGTH: usize = 28;
const AES_256_LENGTH: usize = 32;

#[derive(Debug)]
pub enum DecryptError {
	InvalidLength,
}

fn base64_decode<T: AsRef<[u8]>>(binary: T) -> Result<Vec<u8>, base64::DecodeError> {
	let binary = binary.as_ref();
	let mut buf = vec![0; binary.len() * 4 / 3 + 4];

	let bytes_written = base64::decode_config_slice(binary, base64::STANDARD, &mut buf)?;

	buf.resize(bytes_written, 0);
	Ok(buf)
}

fn batojs_parse(batojs: String) -> String {
	let preprocess = batojs
		.replace(
			"+(+(+!+[]+[+!+[]]+(!![]+[])[!+[]+!+[]+!+[]]+[!+[]+!+[]]+[+[]])+[])[+!+[]]+",
			".",
		)
		.replace("+[]", "!")
		.chars()
		.collect::<Vec<char>>();
	let mut ret = String::new();
	let mut tmp = 0;
	let mut i: usize = 0;
	while i < preprocess.len() {
		if preprocess[i] == '!' && preprocess[i + 1] == '!' {
			tmp += 1;
			i += 1;
		} else if preprocess[i] == ']' {
			ret.push_str(format!("{tmp}").as_str());
			tmp = 0;
		} else if preprocess[i] == '.' {
			ret.push('.');
		}
		i += 1;
	}
	ret
}

/// A function that imitates basic usage of CryptoJS.AES.decrypt(message, key, {
/// iv: iv }).
pub fn cryptojs_aes_decrypt(
	message: &[u8],
	key: &[u8],
	iv: Option<&[u8; 16]>,
) -> Result<Vec<u8>, DecryptError> {
	let mut key_iv = [0; 48];
	let iv = iv.unwrap_or(&[0; 16]);

	// Check for salt
	let (salt, ciphertext) = if &message[0..=7] == b"Salted__" {
		(&message[8..=15], &message[16..])
	} else {
		("".as_bytes(), message)
	};

	let (actual_key, actual_iv) = if key.len() != AES_128_LENGTH
		&& key.len() != AES_192_LENGTH
		&& key.len() != AES_256_LENGTH
	{
		evpkdf::<Md5>(key, salt, 1, &mut key_iv);
		key_iv.split_at(32)
	} else {
		(key as &[u8], iv as &[u8])
	};
	if let Ok(cipher) = Aes256CbcDec::new_from_slices(actual_key, actual_iv) {
		let pt = cipher.decrypt_padded_vec_mut::<Pkcs7>(ciphertext).unwrap();
		Ok(pt)
	} else {
		Err(DecryptError::InvalidLength)
	}
}

pub fn batojs_decrypt(server: String, batojs: String) -> String {
	let temp = batojs_parse(batojs);
	if let Ok(server_decoded) = base64_decode(server) {
		let pt = cryptojs_aes_decrypt(&server_decoded, temp.as_bytes(), None).unwrap();
		String::from_utf8_lossy(&pt).replace('"', "")
	} else {
		String::new()
	}
}
