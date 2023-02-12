//! no_std unpacker for JavaScript code compressed by [packer](http://dean.edwards.name/packer/).
//!
//! Source code of packer can be found [here](https://github.com/evanw/packer/blob/master/packer.js).
//!
//! This is a port of Tachiyomi's [`:lib-unpacker`](https://github.com/tachiyomiorg/tachiyomi-extensions/blob/master/lib/unpacker/src/main/java/eu/kanade/tachiyomi/lib/unpacker/Unpacker.kt);
use aidoku::std::{String, Vec};

use crate::substring::*;

pub fn unpack<T: AsRef<str>>(packed: T) -> String {
	let mut packed = String::from(packed.as_ref());
	packed = packed
		.substring_after("}('")
		.substring_before(".split('|'),0,{}))")
		.replace("\\'", "\"");

	let data = packed.substring_before("',");
	if data.is_empty() {
		return String::new();
	}

	let dict_str = packed
		.substring_after("',")
		.substring_after('\'')
		.substring_before('\'');
	let dictionary = dict_str.split('|').collect::<Vec<_>>();
	let len = dictionary.len();

	let mut accum = String::new();
	let mut ret = String::new();
	for char in data.chars() {
		if char.is_ascii_alphanumeric() {
			accum.push(char)
		} else {
			if !accum.is_empty() {
				let index = parse_radix_62(&accum);

				if index >= len || dictionary[index].is_empty() {
					ret.push_str(&accum);
				} else {
					ret.push_str(dictionary[index]);
				}
				accum.clear();
			}
			ret.push(char);
		}
	}
	ret
}

fn parse_radix_62<T: AsRef<str>>(data: T) -> usize {
	let data = data.as_ref();
	let mut result = 0;
	for char in data.chars() {
		result = result * 62
			+ match char {
				..='9' => char as usize - '0' as usize,
				'a'.. => char as usize - ('a' as usize - 10),
				_ => char as usize - ('A' as usize - 36),
			}
	}
	result
}
