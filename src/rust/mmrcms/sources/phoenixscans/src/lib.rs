#![no_std]
use aidoku::{MangaContentRating, MangaViewer};
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
	MMRCMSSource {
		base_url: "https://phoenix-scans.pl",
		lang: "pl",
		category: "Kategorii",
		tags: "Tagów",
		detail_categories: "Tagi",
		detail_tags: "Tagi",
		detail_description: "Opis",
		detail_status_ongoing: "Prace w toku",
		detail_status_complete: "Zakończone",
		detail_status_hiatus: "Wstrzymane",
		detail_status_cancelled: "Porzucone",
		category_mapper: |idx| {
			match idx {
				0 => String::new(),
				1..=21 => format!("{}", idx + 32),
				22..=30 => format!("{}", idx + 34),
				31..=32 => format!("{}", idx + 35),
				_ => String::new(),
			}
		},
		category_parser: |_, categories| {
			let mut nsfw = MangaContentRating::Safe;
			let mut viewer = MangaViewer::Rtl;
			for category in categories {
				match category.as_str() {
					// Mature
					"Dojrzałe" | "Hentai" => nsfw = MangaContentRating::Nsfw,
					"Webtoon" | "Manhwa" | "Manhua" => viewer = MangaViewer::Scroll,
					_ => continue,
				}
			}
			(nsfw, viewer)
		},
		tags_mapper: |idx| {
			String::from(match idx {
				1 => "aktywne", // Active
				2 => "zakonczone", // Completed
				3 => "porzucone", // Dropped
				4 => "zawieszone", // Suspended
				5 => "zlicencjonowane", // zlicencjonowane
				_ => "",
			})
		},
		..Default::default()
	}
}
