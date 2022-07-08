#![no_std]
use aidoku::{MangaContentRating, MangaViewer};
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
	MMRCMSSource {
		base_url: "https://readcomicsonline.ru",
		manga_path: "comic",
		category_parser: |_, categories| {
			let mut nsfw = MangaContentRating::Safe;
			let mut viewer = MangaViewer::Ltr;
			for category in categories {
				match category.as_str() {
					"Adult" | "Smut" | "Mature" | "18+" => nsfw = MangaContentRating::Nsfw,
					"Webtoon" | "Manhwa" | "Manhua" => viewer = MangaViewer::Scroll,
					_ => continue,
				}
			}
			(nsfw, viewer)
		},
		category_mapper: |idx| {
			String::from(match idx {
				1 => "17", // One Shots & TPBs
				2 => "33", // DC Comics
				3 => "34", // Marvel Comics
				4 => "35", // Boom Studios
				5 => "36", // Dynamite
				6 => "37", // Rebellion
				7 => "38", // Dark Horse
				8 => "39", // IDW
				9 => "40", // Archie
				10 => "41", // Graphic India
				11 => "42", // Darby Pop
				12 => "43", // Oni Press
				13 => "44", // Icon Comics
				14 => "45", // United Plankton
				15 => "46", // Udon
				16 => "47", // Image Comics
				17 => "48", // Valiant
				18 => "49", // Vertigo
				19 => "50", // Devils Due
				20 => "51", // Aftershock Comics
				21 => "52", // Antartic Press
				22 => "53", // Action Lab
				23 => "54", // American Mythology
				24 => "55", // Zenescope
				25 => "56", // Top Cow
				26 => "57", // Hermes Press
				27 => "58", // 451
				28 => "59", // Black Mask
				29 => "60", // Chapterhouse Comics
				30 => "61", // Red 5
				31 => "62", // Heavy Metal
				32 => "63", // Bongo
				33 => "64", // Top Shelf
				34 => "65", // Bubble
				35 => "66", // Boundless
				36 => "67", // Avatar Press
				37 => "68", // Space Goat Productions
				38 => "69", // BroadSword Comics
				39 => "70", // AAM-Markosia
				40 => "71", // Fantagraphics
				41 => "72", // Aspen
				42 => "73", // American Gothic Press
				43 => "74", // Vault
				44 => "75", // 215 Ink
				45 => "76", // Abstract Studio
				46 => "77", // Albatross
				47 => "78", // ARH Comix
				48 => "79", // Legendary Comics
				49 => "80", // Monkeybrain
				50 => "81", // Joe Books
				51 => "82", // MAD
				52 => "83", // Comics Experience
				53 => "84", // Alterna Comics
				54 => "85", // Lion Forge
				55 => "86", // Benitez
				56 => "87", // Storm King
				57 => "88", // Sucker
				58 => "89", // Amryl Entertainment
				59 => "90", // Ahoy Comics
				60 => "91", // Mad Cave
				61 => "92", // Coffin Comics
				62 => "93", // Magnetic Press
				63 => "94", // Ablaze
				64 => "95", // Europe Comics
				65 => "96", // Humanoids
				66 => "97", // TKO
				67 => "98", // Soleil
				68 => "99", // SAF Comics
				69 => "100", // Scholastic
				70 => "101", // Upshot
				71 => "102", // Stranger Comics
				72 => "103", // Inverse
				73 => "104", // Virus
				_ => "",
			})
		},
		..Default::default()
	}
}
