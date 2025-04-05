use aidoku::{
	helpers::uri::QueryParameters,
	prelude::format,
	std::{
		current_date,
		defaults::{defaults_get, defaults_set},
		ObjectRef, String,
	},
	Filter, FilterType, MangaStatus,
};
use alloc::{borrow::ToOwned, string::ToString, vec::Vec};
extern crate alloc;

pub fn search(filters: Vec<Filter>) -> String {
	let mut query = QueryParameters::new();
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(x) = filter.value.as_string() {
					query.push("q", Some(&x.read()));
				}
			}
			FilterType::Check => {
				match filter.object.get("id").as_string().unwrap().read().as_str() {
					// Status
					"ongoing" => {
						query.push("status[]", Some("1"));
					}
					"completed" => {
						query.push("status[]", Some("2"));
					}
					"announce" => {
						query.push("status[]", Some("3"));
					}
					"halted" => {
						query.push("status[]", Some("4"));
					}
					"ended" => {
						query.push("status[]", Some("5"));
					}

					// Type of manga
					"manga" => {
						query.push("types[]", Some("1"));
					}
					"manhwa" => {
						query.push("types[]", Some("5"));
					}
					"rumanga" => {
						query.push("types[]", Some("8"));
					}
					"oelmanga" => {
						query.push("types[]", Some("4"));
					}
					"manhua" => {
						query.push("types[]", Some("6"));
					}
					"comics" => {
						query.push("types[]", Some("9"));
					}
					_ => continue,
				}
			}
			FilterType::Sort => {
				if let Ok(value) = filter.value.as_object() {
					let index = value.get("index").as_int().unwrap_or(0);
					let asc = value.get("ascending").as_bool().unwrap_or(false);
					match index {
						0 => query.push("fields[]", Some("rate")),
						1 => {
							query.push("rate_min", Some("50")); // Idk. It's was at the XHR request
							query.push("sort_by", Some("rate_avg"))
						}
						2 => query.push("sort_by", Some("views")),
						3 => query.push("sort_by", Some("chap_count")),
						4 => query.push("sort_by", Some("last_chapter_at")),
						5 => query.push("sort_by", Some("created_at")),
						6 => query.push("sort_by", Some("name")),
						7 => query.push("sort_by", Some("rus_name")),
						_ => continue,
					}
					if asc {
						query.push("sort_type", Some("asc"))
					}
				}
			}
			_ => continue,
		}
	}

	query.to_string()
}

pub fn id_to_status(id: i64) -> MangaStatus {
	match id {
		1 => MangaStatus::Ongoing,
		2 => MangaStatus::Completed,
		4 => MangaStatus::Hiatus,
		5 => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
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

pub fn display_title() -> String {
	if defaults_get("display_in_eng")
		.and_then(|value| value.as_bool())
		.unwrap_or(false)
	{
		"eng_name".to_owned()
	} else {
		"rus_name".to_owned()
	}
}

pub fn is_logged() -> bool {
	defaults_get("access_token").is_ok()
}

pub fn get_token() -> String {
	format!(
		"Bearer {}",
		defaults_get("access_token")
			.unwrap()
			.as_string()
			.unwrap()
			.read()
	)
}

pub fn save_token(js: ObjectRef) {
	let access_token = js.get("access_token").as_string().unwrap();
	let refresh_token = js.get("refresh_token").as_string().unwrap();
	let expires_in = js.get("expires_in").as_int().unwrap().into();
	defaults_set("access_token", access_token.0);
	defaults_set("refresh_token", refresh_token.0);
	defaults_set("expires_in", expires_in);
	defaults_set("timestamp", (current_date() as i64).into());
}
