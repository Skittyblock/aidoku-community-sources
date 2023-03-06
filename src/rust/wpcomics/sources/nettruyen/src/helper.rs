use aidoku::{
	prelude::format,
	std::Vec,
	std::{current_date, String, StringRef},
	MangaStatus,
};

// MARK: Mappings
pub fn get_tag_id(genre: String) -> String {
	String::from(match genre.as_str() {
		"Action" => "1",
		"Adult" => "2",
		"Adventure" => "3",
		"Anime" => "4",
		"Chuyển Sinh" => "5",
		"Comedy" => "6",
		"Comic" => "7",
		"Cooking" => "8",
		"Cổ Đại" => "9",
		"Doujinshi" => "10",
		"Drama" => "11",
		"Đam Mỹ" => "12",
		"Ecchi" => "13",
		"Fantasy" => "14",
		"Gender Bender" => "15",
		"Harem" => "16",
		"Lịch sử" => "17",
		"Horror" => "18",
		"Josei" => "20",
		"Live action" => "21",
		"Manga" => "23",
		"Manhua" => "24",
		"Manhwa" => "25",
		"Martial Arts" => "26",
		"Mature" => "27",
		"Mecha" => "28",
		"Mystery" => "30",
		"Ngôn Tình" => "32",
		"One shot" => "33",
		"Psychological" => "34",
		"Romance" => "35",
		"School Life" => "36",
		"Sci-fi" => "37",
		"Seinen" => "38",
		"Shoujo" => "39",
		"Shoujo Ai" => "40",
		"Shounen" => "41",
		"Shounen Ai" => "42",
		"Slice of Life" => "43",
		"Smut" => "44",
		"Soft Yaoi" => "45",
		"Soft Yuri" => "46",
		"Sports" => "47",
		"Supernatural" => "48",
		"Tạp chí truyện tranh" => "49",
		"Thiếu Nhi" => "50",
		"Tragedy" => "51",
		"Trinh Thám" => "52",
		"Truyện Màu" => "53",
		"Truyện scan" => "54",
		"Việt Nam" => "55",
		"Webtoon" => "56",
		"Xuyên Không" => "57",
		"Yaoi" => "58",
		"Yuri" => "59",
		"16+" => "60",
		"18+" => "61",
		_ => "",
	})
}

pub fn status_map(arg1: String) -> MangaStatus {
	return match arg1.as_str() {
		"Đang tiến hành" => MangaStatus::Ongoing,
		"Đã hoàn thành" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};
}

// MARK: Other utilities
#[allow(clippy::too_many_arguments)]
pub fn get_search_url(
	base_url: String,
	query: String,
	page: i32,
	include: Vec<String>,
	exclude: Vec<String>,
	sort_by: i32,
	gender: i32,
	completed: i32,
	chapter_count: i32,
) -> String {
	if !query.is_empty() {
		format!("{base_url}/tim-truyen?page={page}&keyword={query}")
	} else {
		format!(
			"{base_url}/tim-truyen-nang-cao?genres={}&notgenres={}&gender={gender}&status={completed}&minchapter={chapter_count}&sort={sort_by}&page={page}",
			include.join(","),
			exclude.join(",")
		)
	}
}

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
