use core::{iter::Map, ops::{Index, IndexMut}};

use aidoku::{std::{String, Vec, format}, prelude::format};
use alloc::{vec, string::ToString};

pub struct Decoder {
	func: String,
	a: i32,
	c: i32,
	data: Vec<String>,
}

impl Decoder {
	pub fn new(document: String) -> Self {
		// aidoku::prelude::println!("document: {}", document);

		let script = get_script(document);

		let func = get_func(script.clone());
		let a = get_a(script.clone(), func.clone()).parse::<i32>().unwrap();
		let c = get_c(script.clone(), func.clone()).parse::<i32>().unwrap();
		let data: Vec<String> = get_data(script.clone(), func.clone());

		aidoku::prelude::println!("data: {:?}", data);

		Decoder { func, a, c, data }
	}

	fn e(&self, c: i32) -> String {
		let mut prefix: String = String::new();
		if (c >= self.a) {prefix = self.e(c / self.a);}

        let _vec = vec![self.tr(c % self.a, 36), String::from_utf8(vec![(c % self.a + 29) as u8]).unwrap()];
        let mut _index =  0; 
        if c % self.a > 35 {_index = 1;}
        let suffix = _vec[_index].clone();

        return format!("{}{}", prefix, suffix);
	}

    fn tr(&self, value: i32, num: i32) -> String {
        let tmp = self.itr(value, num);
        if tmp.eq("") { return String::from("0"); }
        return tmp;
    }

    fn itr(&self, value: i32, num: i32) -> String {
        let d = String::from("0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ");
        if value <= 0 {
            return String::from("");
        }
        let first = self.itr(value / num, num);
        let second = d.chars().nth((value % num) as usize).unwrap();
        return format!("{}{}", first, second);
    }

	pub fn decode(&self) -> String {
        let mut c = (self.c - 1).clone();
        let mut d_key: Vec<String> = vec![];
        let mut d_value: Vec<String> = vec![];
        while c > -1 {
            let key = self.e(c);
            let mut _value_index = 0;
            if self.data[c as usize].eq("") {
                _value_index = 1;
            }
            let value = vec![self.data[c as usize].clone(), self.e(c)][_value_index].clone();

            let index = d_key.clone().iter().position(|r| r.eq(key.as_str())).unwrap_or(999);
            if index == 999 {
                d_key.push(key);
                d_value.push(value);
            } else {
                d_key[index] = key;
                d_value[index] = value;
            }
            c -= 1;
        }

        aidoku::prelude::println!("d_key: {:?}", d_key);
        aidoku::prelude::println!("d_value: {:?}", d_value);

        let mut func = self.func.clone();
        let mut result: Vec<String> = vec![];
        for c in func.split("") {
            let char = String::from(c);
            if d_key.contains(&char) {
                let index = d_key.clone().iter().position(|r| r.eq(char.as_str())).unwrap();
                result.push(d_value[index].clone());
            } else {
                result.push(char);
            }
        }

        aidoku::prelude::println!("func: {:?}", func);
        aidoku::prelude::println!("result: {:?}", result.join(""));

        return String::new();
	}
}

fn get_script(document: String) -> String {
	let mut script: String = String::new();
	for splited in document.split("window[\"\\x65\\x76\\x61\\x6c\"]") {
		if splited.starts_with("(function") {
			for s in splited.split(" </script>") {
				if s.starts_with("(function") {
					script.push_str(s);
				}
			}
		}
	}

	return script;
}

fn get_func(script: String) -> String {
	let mut func: String = String::new();
	for splited in script.split(";return p;}") {
		if splited.starts_with("('") {
			for s in splited.split("',") {
				if s.starts_with("(") {
					func.push_str(s.replacen("('", "", 1).as_str());
				}
			}
		}
	}

	return func;
}

fn get_a(script: String, func: String) -> String {
	let mut a: String = String::new();
	for splited in script.split(";return p;}") {
		if splited.starts_with("('") {
			let s = splited.replace(func.as_str(), "func");
			let mut index = 0;
			for ss in s.split(",") {
				if index == 1 {
					a.push_str(ss);
					break;
				}
				index += 1;
			}
		}
	}

	return a;
}

fn get_c(script: String, func: String) -> String {
	let mut c: String = String::new();
	for splited in script.split(";return p;}") {
		if splited.starts_with("('") {
			let s = splited.replace(func.as_str(), "func");
			let mut index = 0;
			for ss in s.split(",") {
				if index == 2 {
					c.push_str(ss);
					break;
				}
				index += 1;
			}
		}
	}

	return c;
}

fn get_data(script: String, func: String) -> Vec<String> {
	let mut data: Vec<String> = Vec::new();
	let mut dataStr: String = String::new();
	for splited in script.split(";return p;}") {
		if splited.starts_with("('") {
			let s = splited.replace(func.as_str(), "func");
			let mut i = 0;
			for ss in s.split(",") {
				if i == 3 {
					let mut j = 0;
					for sss in ss.split("'") {
						if j == 1 {
							dataStr.push_str(sss);
						}
						j += 1;
					}
					break;
				}
				i += 1;
			}
		}
	}

	dataStr = String::from_utf16(&decompress_from_base64(dataStr.as_str()).unwrap()).unwrap();
	for str in dataStr.split("|") {
		data.push(String::from(str));
	}
	return data;
}

// LZ String
const URI_KEY: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+-$";
const BASE64_KEY: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
const U8_CODE: u8 = 0;
const U16_CODE: u8 = 1;
const CLOSE_CODE: u8 = 2;
const START_CODE_BITS: u8 = 2;

#[derive(Debug)]
pub struct DecompressContext<I> {
	val: u16,
	compressed_data: I,
	position: u16,
	reset_val: u16,
}

impl<I> DecompressContext<I>
where
	I: Iterator<Item = u16>,
{
	#[inline]
	pub fn new(mut compressed_data: I, bits_per_char: u8) -> Option<Self> {
		let reset_val_pow = bits_per_char - 1;
		let reset_val: u16 = 1 << reset_val_pow;

		Some(DecompressContext {
			val: compressed_data.next()?,
			compressed_data,
			position: reset_val,
			reset_val,
		})
	}

	#[inline]
	pub fn read_bit(&mut self) -> Option<bool> {
		let res = self.val & self.position;
		self.position >>= 1;

		if self.position == 0 {
			self.position = self.reset_val;
			self.val = self.compressed_data.next()?;
		}

		Some(res != 0)
	}

	#[inline]
	pub fn read_bits(&mut self, n: u8) -> Option<u32> {
		let mut res = 0;
		let max_power: u32 = 1 << n;
		let mut power: u32 = 1;
		while power != max_power {
			res |= u32::from(self.read_bit()?) * power;
			power <<= 1;
		}

		Some(res)
	}
}

fn decompress_from_base64(compressed: &str) -> Option<Vec<u16>> {
	let compressed: Option<Vec<u16>> = compressed
		.encode_utf16()
		.flat_map(|c| {
			BASE64_KEY
				.iter()
				.position(|k| u8::try_from(c) == Ok(*k))
				.map(|n| u16::try_from(n).ok())
		})
		.collect();

	decompress_internal(compressed?.into_iter(), 6)
}

fn decompress_internal<I>(compressed: I, bits_per_char: u8) -> Option<Vec<u16>>
where
	I: Iterator<Item = u16>,
{
	let mut ctx = match DecompressContext::new(compressed, bits_per_char) {
		Some(ctx) => ctx,
		None => return Some(Vec::new()),
	};

	let mut dictionary: Vec<Vec<u16>> = Vec::with_capacity(16);
	for i in 0_u16..3_u16 {
		dictionary.push(vec![i]);
	}

	// u8::MAX > u2::MAX
	let code = u8::try_from(ctx.read_bits(START_CODE_BITS)?).unwrap();
	let first_entry = match code {
		U8_CODE | U16_CODE => {
			let bits_to_read = (code * 8) + 8;
			// bits_to_read == 8 or 16 <= 16
			u16::try_from(ctx.read_bits(bits_to_read)?).unwrap()
		}
		CLOSE_CODE => return Some(Vec::new()),
		_ => return None,
	};
	dictionary.push(vec![first_entry]);

	let mut w = vec![first_entry];
	let mut result = vec![first_entry];
	let mut num_bits: u8 = 3;
	let mut enlarge_in: u64 = 4;
	let mut entry;
	loop {
		let mut code = ctx.read_bits(num_bits)?;
		match u8::try_from(code) {
			Ok(code_u8 @ (U8_CODE | U16_CODE)) => {
				let bits_to_read = (code_u8 * 8) + 8;
				// if cc == 0 {
				// if (errorCount++ > 10000) return "Error"; // TODO: Error logic
				// }

				// bits_to_read == 8 or 16 <= 16
				let bits = u16::try_from(ctx.read_bits(bits_to_read)?).unwrap();
				dictionary.push(vec![bits]);
				code = u32::try_from(dictionary.len() - 1).ok()?;
				enlarge_in -= 1;
			}
			Ok(CLOSE_CODE) => return Some(result),
			_ => {}
		}

		if enlarge_in == 0 {
			enlarge_in = 1 << num_bits;
			num_bits += 1;
		}

		// Return error if code cannot be converted to dictionary index
		let code_usize = usize::try_from(code).ok()?;
		if let Some(entry_value) = dictionary.get(code_usize) {
			entry = entry_value.clone();
		} else if code_usize == dictionary.len() {
			entry = w.clone();
			entry.push(*w.first()?);
		} else {
			return None;
		}

		result.extend(&entry);

		// Add w+entry[0] to the dictionary.
		let mut to_be_inserted = w.clone();
		to_be_inserted.push(*entry.first()?);
		dictionary.push(to_be_inserted);
		enlarge_in -= 1;

		w = entry;

		if enlarge_in == 0 {
			enlarge_in = 1 << num_bits;
			num_bits += 1;
		}
	}
}
