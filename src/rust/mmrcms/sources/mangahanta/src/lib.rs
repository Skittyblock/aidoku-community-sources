#![no_std]
use aidoku::{MangaContentRating, MangaViewer};
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
	MMRCMSSource {
		base_url: "http://mangahanta.com",
		lang: "tr",
		category: "Kategori",
		category_parser: |_, categories| {
			let mut nsfw = MangaContentRating::Safe;
			let mut viewer = MangaViewer::Rtl;
			for category in categories {
				match category.trim() {
					// Adult
					"Yetişkin" => {
						nsfw = MangaContentRating::Nsfw
					}
					"Ecchi" => {
						nsfw = match nsfw {
							MangaContentRating::Nsfw => MangaContentRating::Nsfw,
							_ => MangaContentRating::Suggestive,
						}
					}
					"Webtoon" | "Manhwa" | "Manhua" => viewer = MangaViewer::Scroll,
					_ => continue,
				}
			}
			(nsfw, viewer)
		},
		..Default::default()
	}
}
