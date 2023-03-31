use aidoku::{
	prelude::format, std::html::Node, std::String, std::Vec, MangaContentRating, MangaViewer,
};

pub fn trunc_trailing_comic(title: String) -> String {
	let temp = title.chars().rev().collect::<String>();
	if temp.find("cimoC") == Some(0) {
		return temp
			.replacen("cimoC", "", 1)
			.chars()
			.rev()
			.collect::<String>();
	} else {
		return temp.chars().rev().collect::<String>();
	}
}

pub fn extract_f32_from_string(title: String, text: String) -> Vec<f32> {
	text.replace(&title, "")
		.chars()
		.filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.' || *a == '+')
		.collect::<String>()
		.split(' ')
		.collect::<Vec<&str>>()
		.into_iter()
		.map(|a| a.parse::<f32>().unwrap_or(-1.0))
		.filter(|a| *a >= 0.0)
		.collect::<Vec<f32>>()
}

pub fn append_protocol(url: String) -> String {
	if !url.starts_with("http") {
		format!("{}{}", "https:", url)
	} else {
		url
	}
}

pub fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if curr.is_ascii_alphanumeric() {
			result.push(curr);
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}

	String::from_utf8(result).unwrap_or_default()
}

pub fn get_tag_id(genre: i64) -> String {
	String::from(match genre {
		1 => "marvel",
		2 => "dc-comics",
		3 => "action",
		4 => "adventure",
		5 => "anthology",
		6 => "anthropomorphic",
		7 => "biography",
		8 => "children",
		9 => "comedy",
		10 => "crime",
		11 => "cyborgs",
		12 => "dark-horse",
		13 => "demons",
		14 => "drama",
		15 => "fantasy",
		16 => "family",
		17 => "fighting",
		18 => "gore",
		19 => "graphic-novels",
		20 => "historical",
		21 => "horror",
		22 => "leading-ladies",
		23 => "literature",
		24 => "magic",
		25 => "manga",
		26 => "martial-arts",
		27 => "mature",
		28 => "mecha",
		29 => "military",
		30 => "movie-cinematic-link",
		31 => "mystery",
		32 => "mythology",
		33 => "psychological",
		34 => "personal",
		35 => "political",
		36 => "post-apocalyptic",
		37 => "pulp",
		38 => "robots",
		39 => "romance",
		40 => "sci-fi",
		41 => "slice-of-life",
		42 => "science-fiction",
		43 => "sport",
		44 => "spy",
		45 => "superhero",
		46 => "supernatural",
		47 => "suspense",
		48 => "thriller",
		49 => "vampires",
		50 => "vertigo",
		51 => "video-games",
		52 => "war",
		53 => "western",
		54 => "zombies",
		_ => "",
	})
}

pub fn text_with_newlines(node: Node) -> String {
	let html = node.html().read();
	if !String::from(html.trim()).is_empty() {
		if let Ok(node) = Node::new_fragment(
			node.html()
				.read()
				.replace("<br>", "{{ .LINEBREAK }}")
				.as_bytes(),
		) {
			node.text().read().replace("{{ .LINEBREAK }}", "\n")
		} else {
			String::new()
		}
	} else {
		String::new()
	}
}

pub fn category_parser(
	categories: &Vec<String>,
	default_nsfw: MangaContentRating,
	default_viewer: MangaViewer,
) -> (MangaContentRating, MangaViewer) {
	let mut nsfw = default_nsfw;
	let mut viewer = default_viewer;
	for category in categories {
		match category.as_str() {
			"Smut" | "Mature" | "18+" => nsfw = MangaContentRating::Nsfw,
			"Ecchi" | "16+" => {
				nsfw = match nsfw {
					MangaContentRating::Nsfw => MangaContentRating::Nsfw,
					_ => MangaContentRating::Suggestive,
				}
			}
			"Webtoon" | "Manhwa" | "Manhua" => viewer = MangaViewer::Scroll,
			_ => continue,
		}
	}
	(nsfw, viewer)
}
