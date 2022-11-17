use aidoku::{
	error::Result,
	helpers::uri::encode_uri_component,
	prelude::format,
	std::{
		net::{HttpMethod, Request},
		String, Vec,
	},
	Page,
};
use alloc::{borrow::ToOwned, vec};

pub mod en;
pub mod es;
pub mod fr;
pub mod ko;
pub mod ru;
pub mod zh;

static THUMBNAIL_URL: &str = "https://fakeimg.pl/550x780/ffffff/6e7b91/?font=museo&text=xkcd";

#[derive(PartialEq, Eq)]
enum ImageVariant {
	Latin,
	Cjk,
}

trait ToImageUrl {
	fn to_image_url(&self, variant: ImageVariant) -> String;
}

impl<S> ToImageUrl for S
where
	S: AsRef<str>,
{
	fn to_image_url(&self, variant: ImageVariant) -> String {
		let text = encode_uri_component(self.as_ref());
		match variant {
            ImageVariant::Latin => format!("https://fakeimg.pl/1500x2126/ffffff/000000/?font=noto&font_size=42&text={text}"),
            ImageVariant::Cjk => format!("https://placehold.jp/42/ffffff/000000/1500x2126.png?css=%7B%22padding%22%3A%22%200%20300px%22%2C%22text-align%22%3A%22left%22%7D&text={text}"),
        }
	}
}

fn word_wrap<T: AsRef<str>>(title: T, alt: T) -> String {
	let title = title.as_ref();
	let alt = alt.as_ref();
	let mut ret = String::new();
	title.split(' ').enumerate().for_each(|(idx, val)| {
		if idx != 0 && idx % 7 == 0 {
			ret.push('\n');
		}
		ret.push_str(val);
		ret.push(' ');
	});
	ret.push_str("\n\n");

	let mut char_count = 0;
	alt.replace("\r\n", " ").split(' ').for_each(|val| {
		if char_count > 25 {
			ret.push('\n');
			char_count = 0;
		}
		ret.push_str(val);
		ret.push(' ');
		char_count += val.len() + 1;
	});

	ret
}

fn get_page_list<T: AsRef<str>>(
	url: T,
	selector: T,
	interactive_if_empty: bool,
	open_in_browser_message: T,
	variant: ImageVariant,
) -> Result<Vec<Page>> {
	let html = Request::new(url, HttpMethod::Get).html()?;
	let node = html.select(selector);
	if (!interactive_if_empty && node.first().next().is_some())
		|| (interactive_if_empty && node.array().is_empty())
	{
		Ok(vec![Page {
			index: 0,
			url: open_in_browser_message.to_image_url(variant),
			..Default::default()
		}])
	} else {
		let url = if node.has_attr("srcset") {
			let raw = node.attr("abs:srcset").read();
			raw.split(' ').next().map(|v| v.to_owned()).unwrap_or(raw)
		} else {
			node.attr("abs:src").read()
		};
		let alt = word_wrap(node.attr("alt").read(), node.attr("title").read());
		Ok(vec![
			Page {
				index: 0,
				url,
				..Default::default()
			},
			Page {
				index: 1,
				url: alt.to_image_url(variant),
				..Default::default()
			},
		])
	}
}
