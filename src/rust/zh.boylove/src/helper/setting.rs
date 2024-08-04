use super::url::{Charset, Url};
use aidoku::std::defaults::defaults_get;

pub fn change_charset() {
	let charset = defaults_get("isTC")
		.and_then(|value| {
			if value.as_bool()? {
				return Ok(Charset::Traditional);
			}

			Ok(Charset::Simplified)
		})
		.unwrap_or_default();

	Url::Charset { charset }.get().send();
}
