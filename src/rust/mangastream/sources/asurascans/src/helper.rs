use aidoku::{
    std::{String, defaults::defaults_get},
};

pub fn get_base_url() -> String {
    let mut base_url:&str ="";
    if let Ok(language) = defaults_get("language").as_string() {
        match language.read().as_str() {
            "tr" => base_url = "https://asurascanstr.com",
            _ =>  base_url = "https://asurascans.com",
        }
    }
    return String::from(base_url);
}

pub fn get_lang_code() -> String {
    let mut code= String::new();
    if let Ok(language) = defaults_get("language").as_string() {
        code = language.read();
    }
    return code;
}
