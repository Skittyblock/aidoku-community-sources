use aidoku::std::{defaults::defaults_get, String};

pub fn get_base_url() -> String {
	let code = get_lang_code();
	match code.as_str() {
		"ar" => String::from("https://ar.flamescans.org"),
		_ => String::from("https://flamescans.org"),
	}
}

pub fn get_lang_code() -> String {
	let mut code = String::new();
	if let Ok(languages) = defaults_get("languages").as_array() {
		if let Ok(language) = languages.get(0).as_string() {
			code = language.read();
		}
	}
	code
}

pub fn get_tag_id(tag: String) -> String {
	let id = match tag.as_str() {
		"Action" => "17",
		"Adventure" => "11",
		"Apocalypse" => "55",
		"Betrayal" => "190",
		"Calm Protagonist" => "191",
		"Comedy" => "45",
		"Coming Soon" => "85",
		"Cultivation" => "110",
		"Dragons" => "193",
		"Drama" => "26",
		"Dungeons" => "22",
		"Ecchi" => "19",
		"Fantasy" => "131",
		"Fusion Fantasy" => "88",
		"Games" => "183",
		"Harem" => "36",
		"Historical" => "44",
		"Horror" => "67",
		"Hunter" => "215",
		"Isekai" => "36",
		"Josei" => "43",
		"Leveling" => "170",
		"Magic" => "13",
		"Martial Arts" => "29",
		"Mature" => "98",
		"Military" => "136",
		"Monster" => "10",
		"Murim" => "138",
		"Mystery" => "122",
		"Novel" => "50",
		"Official" => "60",
		"Pokemon" => "96",
		"Post-Apocalyptic" => "168",
		"Psychological" => "81",
		"Pyschological" => "105",
		"Reincarnation" => "38",
		"Revenge" => "125",
		"Romance" => "40",
		"School Life" => "132",
		"Sci-fi" => "97",
		"Seinen" => "77",
		"Shoujo" => "41",
		"Shoujo Ai" => "174",
		"Shounen" => "24",
		"Slice of Life" => "138",
		"Sports" => "28",
		"Supernatural" => "15",
		"Survival" => "133",
		"Sword and Magic" => "192",
		"System" => "122",
		"Thriller" => "82",
		"Time Travel" => "211",
		"Tragedy" => "64",
		"Transmigration" => "185",
		"Video Games" => "29",
		"VR" => "162",
		"Zombies" => "68",
		_ => "",
	};
	String::from(id)
}
