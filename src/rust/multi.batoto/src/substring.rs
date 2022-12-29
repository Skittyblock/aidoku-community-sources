//! Substring utility functions.
use core::str::pattern::{Pattern, ReverseSearcher, Searcher};

pub trait Substring {
	/// Returns a substring before the first occurence of pattern.
	fn substring_before<'a, P: Pattern<'a>>(&'a self, pat: P) -> Option<&'a str>;

	/// Returns a substring before the last occurence of pattern.
	fn substring_before_last<'a, P>(&'a self, pat: P) -> Option<&'a str>
	where
		P: Pattern<'a>,
		<P as Pattern<'a>>::Searcher: ReverseSearcher<'a>;

	/// Returns a substring after the first occurence of pattern.
	fn substring_after<'a, P: Pattern<'a>>(&'a self, pat: P) -> Option<&'a str>;

	/// Returns a substring after the last occurence of pattern.
	fn substring_after_last<'a, P>(&'a self, pat: P) -> Option<&'a str>
	where
		P: Pattern<'a>,
		<P as Pattern<'a>>::Searcher: ReverseSearcher<'a>;
}

impl Substring for str {
	#[inline]
	fn substring_before<'a, P: Pattern<'a>>(&'a self, pat: P) -> Option<&'a str> {
		match self.find(pat) {
			Some(i) => Some(&self[..i]),
			None => None,
		}
	}

	#[inline]
	fn substring_before_last<'a, P>(&'a self, pat: P) -> Option<&'a str>
	where
		P: Pattern<'a>,
		<P as Pattern<'a>>::Searcher: ReverseSearcher<'a>,
	{
		match self.rfind(pat) {
			Some(i) => Some(&self[..i]),
			None => None,
		}
	}

	#[inline]
	fn substring_after<'a, P: Pattern<'a>>(&'a self, pat: P) -> Option<&'a str> {
		match pat.into_searcher(self).next_match() {
			Some((_, end)) => Some(&self[end..]),
			None => None,
		}
	}

	#[inline]
	fn substring_after_last<'a, P>(&'a self, pat: P) -> Option<&'a str>
	where
		P: Pattern<'a>,
		<P as Pattern<'a>>::Searcher: ReverseSearcher<'a>,
	{
		match pat.into_searcher(self).next_match_back() {
			Some((_, end)) => Some(&self[end..]),
			None => None,
		}
	}
}
