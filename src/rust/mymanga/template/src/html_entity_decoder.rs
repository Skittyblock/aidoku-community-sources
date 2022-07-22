// MIT License

// Copyright (c) 2020 magiclen.org (Ron Li)

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
use aidoku::std::{String, Vec};
use alloc::borrow::Cow;

#[inline]
fn write_char_to_vec(c: char, output: &mut Vec<u8>) {
	let width = c.len_utf8();

	output.reserve(width);

	let current_length = output.len();

	unsafe {
		output.set_len(current_length + width);
	}

	c.encode_utf8(&mut output[current_length..]);
}

pub fn decode_html_entities<S: ?Sized + AsRef<str>>(text: &S) -> Cow<str> {
	let text = text.as_ref();
	let text_bytes = text.as_bytes();
	let text_length = text_bytes.len();

	let mut p = 0;
	let mut ep = 0;
	let mut e;

	let mut step = 0;

	let (mut v, mut start) = loop {
		if p == text_length {
			return Cow::from(text);
		}

		e = text_bytes[p];

		match step {
			0 => {
				if e == b'&' {
					step = 1;
					ep = p;
				}
			}
			1 => {
				match e {
					b'#' => {
						step = 3;
					}
					b';' => {
						// incorrect
						step = 0;
					}
					_ => {
						step = 2;
					}
				}
			}
			2 => {
				if e == b';' {
					// named
					let mut v = Vec::with_capacity(text_length);

					v.extend_from_slice(&text_bytes[..ep]);

					break (v, ep);
				}
			}
			3 => {
				match e {
					b'x' | b'X' => {
						step = 5;
					}
					b';' => {
						// incorrect
						step = 0;
					}
					_ => step = 4,
				}
			}
			4 => {
				if e == b';' {
					// numeric
					let mut v = Vec::with_capacity(text_length);

					v.extend_from_slice(&text_bytes[..ep]);

					let number = unsafe { text.get_unchecked((ep + 2)..p) };

					match number.parse::<u32>() {
						Ok(number) => match char::try_from(number) {
							Ok(c) => {
								write_char_to_vec(c, &mut v);
								break (v, p + 1);
							}
							Err(_) => break (v, ep),
						},
						Err(_) => break (v, ep),
					}
				}
			}
			5 => {
				match e {
					b';' => {
						// incorrect
						step = 0;
					}
					_ => step = 6,
				}
			}
			6 => {
				if e == b';' {
					// hex
					let mut v = Vec::with_capacity(text_length);

					v.extend_from_slice(&text_bytes[..ep]);

					let hex = unsafe { text.get_unchecked((ep + 3)..p) };

					match u32::from_str_radix(hex, 16) {
						Ok(number) => match char::try_from(number) {
							Ok(c) => {
								write_char_to_vec(c, &mut v);
								break (v, p + 1);
							}
							Err(_) => break (v, ep),
						},
						Err(_) => break (v, ep),
					}
				}
			}
			_ => unreachable!(),
		}

		p += 1;
	};

	p += 1;

	step = 0;

	for e in text_bytes[p..].iter().copied() {
		match step {
			0 => {
				if e == b'&' {
					step = 1;
					ep = p;
				}
			}
			1 => {
				match e {
					b'#' => {
						step = 3;
					}
					b';' => {
						// incorrect
						step = 0;
					}
					_ => {
						step = 2;
					}
				}
			}
			2 => {
				if e == b';' {
					// named
					step = 0;
				}
			}
			3 => {
				match e {
					b'x' | b'X' => {
						step = 5;
					}
					b';' => {
						// incorrect
						step = 0;
					}
					_ => step = 4,
				}
			}
			4 => {
				if e == b';' {
					// numeric
					step = 0;

					let number = unsafe { text.get_unchecked((ep + 2)..p) };

					if let Ok(number) = number.parse::<u32>() {
						if let Ok(c) = char::try_from(number) {
							v.extend_from_slice(&text_bytes[start..ep]);
							start = p + 1;
							write_char_to_vec(c, &mut v);
						}
					}
				}
			}
			5 => {
				match e {
					b';' => {
						// incorrect
						step = 0;
					}
					_ => step = 6,
				}
			}
			6 => {
				if e == b';' {
					// hex
					step = 0;

					let hex = unsafe { text.get_unchecked((ep + 3)..p) };

					if let Ok(number) = u32::from_str_radix(hex, 16) {
						if let Ok(c) = char::try_from(number) {
							v.extend_from_slice(&text_bytes[start..ep]);
							start = p + 1;
							write_char_to_vec(c, &mut v);
						}
					}
				}
			}
			_ => unreachable!(),
		}

		p += 1;
	}

	v.extend_from_slice(&text_bytes[start..p]);

	Cow::from(unsafe { String::from_utf8_unchecked(v) })
}
