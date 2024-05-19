use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	prelude::println,
};
use core::fmt::Display;
use regex::Regex as OriginalRegex;

pub struct Regex;

impl Regex {
	#[allow(clippy::new_ret_no_self)]
	pub fn new<S: AsRef<str>>(pat: S) -> Result<OriginalRegex> {
		OriginalRegex::new(pat.as_ref()).map_err(to_aidoku_error)
	}
}

pub fn to_aidoku_error<E: Display>(err: E) -> AidokuError {
	println!("{err}");

	AidokuError {
		reason: AidokuErrorKind::Unimplemented,
	}
}
