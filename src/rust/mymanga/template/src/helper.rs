use aidoku::{
	std::{html::Node, String, Vec},
	MangaContentRating, MangaViewer,
};

pub fn extract_f32_from_string(title: String, text: String) -> Vec<f32> {
	let mut last_char_was_digit: bool = false;
	text.replace(&title, "")
		.chars()
		.filter(|a| {
			if (*a).is_ascii_digit() {
				last_char_was_digit = true;
				return true;
			} else if *a == '.' && last_char_was_digit || *a == '+' || *a == ' ' {
				last_char_was_digit = false;
				return true;
			}
			false
		})
		.collect::<String>()
		.split(' ')
		.filter_map(|a| a.parse::<f32>().ok())
		.collect::<Vec<f32>>()
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
			"Novel" | "VnComic" => viewer = MangaViewer::Ltr,
			_ => continue,
		}
	}
	(nsfw, viewer)
}
