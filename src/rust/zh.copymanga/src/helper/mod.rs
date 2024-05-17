use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	prelude::println,
};
use regex::Regex as OriginalRegex;

pub struct Regex;

impl Regex {
	#[allow(clippy::new_ret_no_self)]
	pub fn new<S: AsRef<str>>(pat: S) -> Result<OriginalRegex> {
		OriginalRegex::new(pat.as_ref()).map_err(|e| {
			println!("{e}");

			AidokuError {
				reason: AidokuErrorKind::Unimplemented,
			}
		})
	}
}
