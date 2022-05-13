use aidoku::{
	std::{String, defaults::defaults_get},
};

pub fn get_base_url() -> String {
	String::from(match get_lang_code().as_str() {
		"tr" => "https://asurascanstr.com",
		_ => "https://asurascans.com",
	})
}

pub fn get_lang_code() -> String {
	let mut code = String::new();
	if let Ok(languages) = defaults_get("languages").as_array() {
		if let Ok(language) = languages.get(0).as_string() {
			code = language.read();
		}
	}
	return code;
}
