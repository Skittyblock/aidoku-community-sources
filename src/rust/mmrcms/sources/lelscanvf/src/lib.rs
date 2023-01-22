#![no_std]
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
	MMRCMSSource {
		base_url: "https://lelscanvf.com",
		lang: "fr",
		category_mapper: |idx| {
			String::from(match idx {
				1 => "1", // Action
				2 => "2", // Adventure
				3 => "3", // Comedy
				4 => "4", // Doujinshi
				5 => "5", // Drama
				6 => "6", // Ecchi
				7 => "7", // Fantasy
				8 => "8", // Gender Bender
				9 => "9", // Harem
				10 => "10", // Historical
				11 => "11", // Horror
				12 => "12", // Josei
				13 => "13", // Martial Arts
				14 => "14", // Mature
				15 => "15", // Mecha
				16 => "16", // Mystery
				17 => "17", // One Shot
				18 => "18", // Psychological
				19 => "19", // Romance
				20 => "20", // School Life
				21 => "21", // Sci-fi
				22 => "22", // Seinen
				23 => "23", // Shoujo
				24 => "24", // Shoujo Ai
				25 => "25", // Shounen
				26 => "26", // Shounen Ai
				27 => "27", // Slice of Life
				28 => "28", // Sports
				29 => "29", // Supernatural
				30 => "30", // Tragedy
				31 => "31", // Yaoi
				32 => "32", // Yuri
				_ => "",
			})
		},
		..Default::default()
	}
}
