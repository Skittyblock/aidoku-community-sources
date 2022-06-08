use aidoku::{
	std::{defaults::defaults_get, html::Node, String, Vec},
	MangaContentRating, MangaViewer,
};
use alloc::string::ToString;

pub fn extract_f32_from_string(title: String, text: String) -> f32 {
	text.replace(&title, "")
		.chars()
		.filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.')
		.collect::<String>()
		.split(' ')
		.collect::<Vec<&str>>()
		.into_iter()
		.map(|a| a.parse::<f32>().unwrap_or(0.0))
		.find(|a| *a > 0.0)
		.unwrap_or(0.0)
}

pub fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789ABCDEF".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if (b'a'..=b'z').contains(&curr)
			|| (b'A'..=b'Z').contains(&curr)
			|| (b'0'..=b'9').contains(&curr)
		{
			result.push(curr);
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}
	String::from_utf8(result).unwrap_or_default()
}

pub fn get_lang_code() -> String {
	let mut code = String::from("vn");
	if let Ok(languages) = defaults_get("languages").as_array() {
		if let Ok(language) = languages.get(0).as_string() {
			code = language.read();
		}
	}
	code
}

pub fn text_with_newlines(node: Node) -> String {
	let html = node.html().read();
	if !String::from(html.trim()).is_empty() {
		Node::new_fragment(
			node.html()
				.read()
				.replace("<br>", "{{ .LINEBREAK }}")
				.as_bytes(),
		)
		.text()
		.read()
		.replace("{{ .LINEBREAK }}", "\n")
	} else {
		String::new()
	}
}

pub fn category_parser(categories: &Vec<String>) -> (MangaContentRating, MangaViewer) {
	let mut nsfw = MangaContentRating::Safe;
	let mut viewer = MangaViewer::Rtl;
	for category in categories {
		match category.as_str() {
			"Adult" | "Smut" | "Mature" | "18+" => nsfw = MangaContentRating::Nsfw,
			"Ecchi" | "16+" => {
				nsfw = match nsfw {
					MangaContentRating::Nsfw => MangaContentRating::Nsfw,
					_ => MangaContentRating::Suggestive,
				}
			}
			"Webtoon" | "Manhwa" | "Manhua" => viewer = MangaViewer::Scroll,
			"VnComic" => viewer = MangaViewer::Ltr,
			_ => continue,
		}
	}
	(nsfw, viewer)
}

pub fn capitalize_first_letter(name: String) -> String {
	let preprocess = name.chars().collect::<Vec<_>>();
	let mut ret = String::with_capacity(preprocess.len() * 2);
	ret.push_str(&preprocess[0].to_uppercase().to_string());
	let mut i: usize = 1;
	while i < preprocess.len() {
		if char::is_ascii_whitespace(&preprocess[i]) {
			ret.push(preprocess[i]);
			ret.push_str(&preprocess[i + 1].to_uppercase().to_string());
			i += 1;
		} else {
			ret.push(preprocess[i]);
		}
		i += 1;
	}
	ret
}
