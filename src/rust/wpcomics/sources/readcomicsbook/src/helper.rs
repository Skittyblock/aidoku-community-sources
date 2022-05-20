use aidoku::{prelude::format, std::String, MangaStatus};

pub fn listing_mapping(listing: String) -> String {
	String::from(match listing.as_str() {
		"Ongoing" => "ongoing-comics",
		"New" => "new-comics",
		"Popular" => "popular-comics",
		_ => "",
	})
}

pub fn status_map(arg1: String) -> MangaStatus {
	return match arg1.as_str() {
		"Ongoing" => MangaStatus::Ongoing,
		"Completed" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};
}

pub fn get_tag_id(genre: i64) -> String {
	return String::from(match genre {
		1 => "marvel",
		2 => "dc-comics",
		3 => "action",
		4 => "adventure",
		5 => "anthology",
		6 => "anthropomorphic",
		7 => "biography",
		8 => "children",
		9 => "comedy",
		10 => "crime",
		11 => "cyborgs",
		12 => "dark-horse",
		13 => "demons",
		14 => "drama",
		15 => "fantasy",
		16 => "family",
		17 => "fighting",
		18 => "gore",
		19 => "graphic-novels",
		20 => "historical",
		21 => "horror",
		22 => "leading-ladies",
		23 => "literature",
		24 => "magic",
		25 => "manga",
		26 => "martial-arts",
		27 => "mature",
		28 => "mecha",
		29 => "military",
		30 => "movie-cinematic-link",
		31 => "mystery",
		32 => "mythology",
		33 => "psychological",
		34 => "personal",
		35 => "political",
		36 => "post-apocalyptic",
		37 => "pulp",
		38 => "robots",
		39 => "romance",
		40 => "sci-fi",
		41 => "slice-of-life",
		42 => "science-fiction",
		43 => "sport",
		44 => "spy",
		45 => "superhero",
		46 => "supernatural",
		47 => "suspense",
		48 => "thriller",
		49 => "vampires",
		50 => "vertigo",
		51 => "video-games",
		52 => "war",
		53 => "western",
		54 => "zombies",
		_ => "",
	});
}

pub fn get_search_url(base_url: String, genre: String, page: i32) -> String {
	if genre.len() > 0 {
		return format!("{base_url}/genre/{genre}?page={page}");
	}
	return format!("{base_url}/comic-updates?page={page}");
}
