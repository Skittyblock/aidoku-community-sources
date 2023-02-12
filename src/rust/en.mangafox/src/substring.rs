use core::str::pattern::{Pattern, Searcher};

pub trait Substring {
	/// Returns a substring before the first occurence of pattern.
	fn substring_before<'a, P: Pattern<'a>>(&'a self, pat: P) -> &'a str;

	/// Returns a substring after the first occurence of pattern.
	fn substring_after<'a, P: Pattern<'a>>(&'a self, pat: P) -> &'a str;
}

impl Substring for str {
	#[inline]
	fn substring_before<'a, P: Pattern<'a>>(&'a self, pat: P) -> &'a str {
		match self.find(pat) {
			Some(i) => &self[..i],
			None => "",
		}
	}

	#[inline]
	fn substring_after<'a, P: Pattern<'a>>(&'a self, pat: P) -> &'a str {
		match pat.into_searcher(self).next_match() {
			Some((_, end)) => &self[end..],
			None => "",
		}
	}
}
