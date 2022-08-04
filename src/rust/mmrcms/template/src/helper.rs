use aidoku::std::String;

pub fn extract_f32_from_string(title: String, text: String) -> f32 {
	text.replace(&title, "")
		.replace(|a: char| a == ',' || a == '_' || a == '-', ".")
		.chars()
		.filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.')
		.collect::<String>()
		.split(' ')
		.map(|a| a.parse::<f32>().unwrap_or(0.0))
		.find(|a| *a > 0.0)
		.unwrap_or(-1.0)
}

pub fn append_protocol(url: String) -> String {
	if !url.starts_with("http") {
		let mut ret = String::from("https:");
		ret.push_str(&url);
		ret
	} else {
		url
	}
}
