use aidoku::{
	prelude::format,
	std::{current_date, String, StringRef, Vec},
};

pub fn convert_time(time_ago: String) -> f64 {
	let current_time = current_date();
	let time_arr = time_ago.split(' ').collect::<Vec<&str>>();
	if time_arr.len() > 2 {
		match time_arr[1] {
			"giây" => current_time - time_arr[0].parse::<f64>().unwrap_or(0.0),
			"phút" => current_time - time_arr[0].parse::<f64>().unwrap_or(0.0) * 60.0,
			"giờ" => current_time - time_arr[0].parse::<f64>().unwrap_or(0.0) * 3600.0,
			"ngày" => current_time - time_arr[0].parse::<f64>().unwrap_or(0.0) * 86400.0,
			_ => current_time,
		}
	} else if *time_arr[0] == time_ago {
		StringRef::from(time_ago)
			.0
			.as_date("dd/MM/yy", Some("en_US"), Some("Asia/Ho_Chi_Minh"))
			.unwrap_or(0.0)
	} else {
		let modified_time = format!(
			"{} {}/{}",
			time_arr[0],
			time_arr[1],
			1970 + (current_time / 31536000.0) as i32
		);
		StringRef::from(modified_time)
			.0
			.as_date("HH:mm dd/MM/yyyy", Some("en_US"), Some("Asia/Ho_Chi_Minh"))
			.unwrap_or(0.0)
	}
}

pub fn genre_mapping(genre: i64) -> String {
	String::from(match genre {
		0 => "",
		1 => "action",
		2 => "adult",
		3 => "adventure",
		4 => "anime",
		5 => "chuyen-sinh",
		6 => "comedy",
		7 => "comic",
		8 => "cooking",
		9 => "co-dai",
		10 => "doujinshi",
		11 => "drama",
		12 => "dam-my",
		13 => "dam-my",
		14 => "ecchi",
		15 => "fantasy",
		16 => "gender-bender",
		17 => "harem",
		18 => "historical",
		19 => "horror",
		20 => "josei",
		21 => "live-action",
		22 => "manga",
		23 => "manhua",
		24 => "manhwa",
		25 => "martial-arts",
		26 => "martial-arts",
		27 => "mature",
		28 => "mecha",
		29 => "mystery",
		30 => "ngon-tinh",
		31 => "one-shot",
		32 => "psychological",
		33 => "romance",
		34 => "school-life",
		35 => "sci-fi",
		36 => "seinen",
		37 => "shoujo",
		38 => "shoujo-ai",
		39 => "shoujo-ai",
		40 => "shounen",
		41 => "shounen-ai",
		42 => "slice-of-life",
		43 => "smut",
		44 => "soft-yaoi",
		45 => "soft-yuri",
		46 => "sports",
		47 => "supernatural",
		48 => "tragedy",
		49 => "xuyen-khong",
		50 => "webtoon",
		51 => "truyen-mau",
		_ => "",
	})
}
