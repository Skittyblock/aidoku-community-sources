use aidoku::{
	std::Vec,
	std::{current_date, String, StringRef},
	MangaStatus,
};
use wpcomics_template::helper::i32_to_string;

// MARK: Mappings
pub fn get_tag_id(genre: String) -> String {
	let id = match genre.as_str() {
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
	};
	return String::from(id);
}

pub fn listing_map(listing: String) -> String {
	let url: &str = match listing.as_str() {
		"Truyện con gái" => "truyen-con-gai",
		"Truyện con trai" => "truyen-con-trai",
		"Hot" => "hot",
		_ => "",
	};
	return String::from(url);
}

pub fn status_map(arg1: String) -> MangaStatus {
	return match arg1.as_str() {
		"Đang tiến hành" => MangaStatus::Ongoing,
		"Đã hoàn thành" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};
}

// MARK: Other utilities
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
	let mut url = String::new();
	url.push_str(&base_url);
	if query.len() > 0 {
		url.push_str("/tim-truyen?page=");
		url.push_str(i32_to_string(page).as_str());
		url.push_str("&keyword=");
		url.push_str(&query);
		return url;
	}
	url.push_str("/tim-truyen-nang-cao?");
	url.push_str("genres=");
	if include.len() > 0 {
		url.push_str(include.join(",").as_str());
	}
	url.push_str("&notgenres=");
	if exclude.len() > 0 {
		url.push_str(exclude.join(",").as_str());
	}
	url.push_str("&gender=");
	url.push_str(i32_to_string(gender).as_str());
	url.push_str("&status=");
	url.push_str(i32_to_string(completed).as_str());
	url.push_str("&minchapter=");
	url.push_str(i32_to_string(chapter_count).as_str());
	url.push_str("&sort=");
	url.push_str(i32_to_string(sort_by).as_str());
	url.push_str("&page=");
	url.push_str(i32_to_string(page).as_str());
	return url;
}

pub fn convert_time(time_ago: String) -> f64 {
	let mut time: f64 = 0.0;
	let current_time = current_date();
	let time_arr = time_ago.split(" ").collect::<Vec<&str>>();
	if time_arr.len() > 2 {
		time = match time_arr[1] {
			"giây" => current_time - time_arr[0].parse::<f64>().unwrap_or(0.0),
			"phút" => current_time - time_arr[0].parse::<f64>().unwrap_or(0.0) * 60.0,
			"giờ" => current_time - time_arr[0].parse::<f64>().unwrap_or(0.0) * 3600.0,
			"ngày" => current_time - time_arr[0].parse::<f64>().unwrap_or(0.0) * 86400.0,
			_ => current_time,
		};
	} else if String::from(time_arr[0]) == time_ago {
		let time_object = StringRef::from(time_ago).0;
		time = time_object
			.as_date("dd/MM/yy", Some("en_US"), Some("Asia/Ho_Chi_Minh"))
			.unwrap_or(0.0);
	} else {
		let current_year: i32 = 1970 + (current_time / 31536000.0) as i32;
		let mut modified_time = String::from(time_arr[0]);
		modified_time.push_str(" ");
		modified_time.push_str(time_arr[1]);
		modified_time.push_str("/");
		modified_time.push_str(&i32_to_string(current_year));
		let time_object = StringRef::from(modified_time).0;
		time = time_object
			.as_date("HH:mm dd/MM/yyyy", Some("en_US"), Some("Asia/Ho_Chi_Minh"))
			.unwrap_or(0.0);
	}
	return time;
}
