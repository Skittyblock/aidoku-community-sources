use crate::BASE_URL;
use aidoku::{
	helpers::uri::QueryParameters,
	prelude::format,
	std::{String, Vec},
	Filter, FilterType,
};
use alloc::string::ToString;

pub fn get_search_url(filters: Vec<Filter>, page: i32) -> String {
	let mut qs = QueryParameters::new();
	qs.push("dou", None);
	qs.push("char", None);
	qs.push("page", Some(&page.to_string()));
	qs.push("search", None);

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(title) = filter.value.as_string() {
					qs.push("name", Some(&title.read()));
				}
			}
			FilterType::Genre => {
				if filter.value.as_int().unwrap_or(-1) == 1 {
					if let Ok(id) = filter.object.get("id").as_string() {
						qs.push("tag[]", Some(&id.read()));
					}
				}
			}
			_ => continue,
		}
	}
	format!("{BASE_URL}/forum/search-plus.php?{qs}")
}
