use aidoku::std::String;

pub fn is_nsfw(genre: String) -> bool {
	let nsfwcategories = "Еччі Юрі Яой 18+"; // maybe new
	if nsfwcategories.contains(&genre) {
		return true;
	}
	//return false;
	false
}

pub fn get_status_string(status: String) -> &'static str {
	match status.as_str() {
		"Триває" => "Ongoing",
		"Закінчений" => "Completed",
		"Невідомо" => "Unknown",
		"Покинуто" => "Cancelled",
		_ => "Unknown",
	}
}

pub fn genres_list() -> [&'static str; 50] {
	[
		"",
		"dementia",
		"boyovik",
		"boyov-mistectva",
		"budenst",
		"vampri",
		"garem",
		"kodomo",
		"detektiv",
		"demons",
		"josei",
		"doujinshi",
		"drama",
		"ecchi",
		"zhahi",
		"gender-bender",
		"games",
		"storia",
		"yonkoma",
		"space",
		"komedia",
		"maho-shoujou",
		"cars",
		"meha",
		"mstika",
		"music",
		"nadprirodne",
		"naukova-fantastika",
		"parody",
		"prigodi",
		"psihologia",
		"police",
		"postapokalptika",
		"romantika",
		"samurai",
		"sentai",
		"seinen",
		"sport",
		"superpower",
		"tragedia",
		"triler",
		"fantastika",
		"fentez",
		"shoujo",
		"shoujo-ai",
		"shounen",
		"shounen-ai",
		"shkola",
		"iur",
		"shonen-ay",
	]
}
