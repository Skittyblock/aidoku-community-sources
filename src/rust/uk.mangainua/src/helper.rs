use aidoku::
{
	std::String,
	MangaContentRating
};

pub fn is_nsfw(genre: String) -> MangaContentRating{

	let nsfwcategories: String = String::from("Еччі Юрі Яой"); // maybe new
	if nsfwcategories.contains(&genre)
	{
		return MangaContentRating::Nsfw;
	}
	//return MangaContentRating::Safe;
	MangaContentRating::Safe
}

pub fn is_nsfwbool(genre: String) -> bool {
	let nsfwcategories: String = String::from("Еччі Юрі Яой"); // maybe new
	if nsfwcategories.contains(&genre)
	{
		return true;
	}
	//return false;
	false
}

pub fn get_status_string(status: String) -> String{
	if status == "Триває"{
		return String::from("Ongoing");
	}
	if status == "Закінчений"{
		return String::from("Completed");
	}
	if status == "Невідомо"{
		return String::from("Unknown");
	}
	if status == "Покинуто"{
		return String::from("Cancelled");
	}
	//return String::from("Unknown"); // find others
	String::from("Unknown") // find others
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
