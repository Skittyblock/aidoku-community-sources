use aidoku::{
	std::String,
	std::{html::Node, Vec},
	MangaContentRating, MangaStatus, MangaViewer,
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

pub fn status_from_string(status: String) -> MangaStatus {
	return match status.as_str() {
		"Đang tiến hành" => MangaStatus::Ongoing,
		"Đã hoàn thành" => MangaStatus::Completed,
		"Tạm ngưng" => MangaStatus::Hiatus,
		"Cancelled" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	};
}

pub fn category_parser(categories: &Vec<String>) -> (MangaContentRating, MangaViewer) {
	let mut nsfw = MangaContentRating::Safe;
	let mut viewer = MangaViewer::Rtl;
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

pub fn genre_map(genre: String) -> String {
	return String::from(match genre.as_str() {
		"16+" => "54",
		"18+" => "45",
		"Action" => "1",
		"Adult" => "2",
		"Adventure" => "3",
		"Anime" => "4",
		"Bạo lực - Máu me" => "67",
		"Comedy" => "5",
		"Comic" => "6",
		"Doujinshi" => "7",
		"Drama" => "49",
		"Ecchi" => "48",
		"Event BT" => "60",
		"Fantasy" => "50",
		"Full màu" => "64",
		"Game" => "61",
		"Gender Bender" => "51",
		"Harem" => "12",
		"Historical" => "13",
		"Horror" => "14",
		"Isekai/Dị giới/Trọng sinh" => "63",
		"Josei" => "15",
		"Live action" => "16",
		"Magic" => "46",
		"manga" => "55",
		"Manhua" => "17",
		"Manhwa" => "18",
		"Martial Arts" => "19",
		"Mature" => "20",
		"Mecha" => "21",
		"Mystery" => "22",
		"Nấu Ăn" => "56",
		"Ngôn Tình" => "65",
		"NTR" => "62",
		"One shot" => "23",
		"Psychological" => "24",
		"Romance" => "25",
		"School Life" => "26",
		"Sci-fi" => "27",
		"Seinen" => "28",
		"Shoujo" => "29",
		"Shoujo Ai" => "30",
		"Shounen" => "31",
		"Shounen Ai" => "32",
		"Slice of life" => "33",
		"Smut" => "34",
		"Soft Yaoi" => "35",
		"Soft Yuri" => "36",
		"Sports" => "37",
		"Supernatural" => "38",
		"Tạp chí truyện tranh" => "39",
		"Tragedy" => "40",
		"Trap (Crossdressing)" => "58",
		"Trinh Thám" => "57",
		"Truyện scan" => "41",
		"Tu chân - tu tiên" => "66",
		"Video Clip" => "53",
		"VnComic" => "42",
		"Webtoon" => "52",
		"Yuri" => "59",
		_ => "",
	});
}
