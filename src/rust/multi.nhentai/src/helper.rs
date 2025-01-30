use aidoku::{
	error::Result,
	std::{ArrayRef, String, Vec},
};

pub fn get_details_url(id: String) -> String {
	let mut string = String::from("https://nhentai.net/api/gallery/");
	string.push_str(&id);
	string
}

pub fn get_file_type(filetype: String) -> String {
	match filetype.as_str() {
		"j" => String::from("jpg"),
		"p" => String::from("png"),
		"w" => String::from("webp"),
		"g" => String::from("gif"),
		_ => String::new(),
	}
}

pub fn get_tag_names_by_type(tags: ArrayRef, tag_type: &str) -> Result<Vec<String>> {
	let mut names: Vec<String> = Vec::new();
	for tag in tags {
		let tag_obj = tag.as_object()?;
		if tag_obj.get("type").as_string()?.read() == tag_type {
			names.push(tag_obj.get("name").as_string()?.read());
		}
	}
	if names.is_empty() {
		names.push(String::new());
	}
	Ok(names)
}

pub fn is_number(s: &str) -> bool {
	s.chars().all(|c| c.is_numeric())
}

pub fn find_media_server(big_string: &str) -> Option<&str> {
	let begin_pattern = r#"window._n_app"#; // finds the script block that contains the media server
	let start_pattern = r#"media_server: "#;
	let end_pattern = r#","#;

	if let Some(begin_index) = big_string.find(begin_pattern) {
		if let Some(start_index) = big_string[begin_index..].find(start_pattern) {
			let start_index = begin_index + start_index + start_pattern.len();
			if let Some(end_index) = big_string[start_index..].find(end_pattern) {
				let end_index = start_index + end_index;
				return Some(&big_string[start_index..end_index]);
			}
		}
	}

	None
}
