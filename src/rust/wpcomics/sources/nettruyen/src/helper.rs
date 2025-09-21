use core::str::FromStr;

use aidoku::{
	prelude::format,
	std::Vec,
	std::{current_date, String, StringRef},
	MangaStatus,
};

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
	category: Option<String>,
	sort_by: i32,
	completed: i32,
) -> String {
	if !query.is_empty() {
		format!("{base_url}/tim-truyen?page={page}&keyword={query}")
	} else {
		format!(
			"{base_url}/tim-truyen/{}&status={completed}&sort={sort_by}&page={page}",
			if let Some(val) = category {
				val
			} else {
				String::new()
			}
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
