#![no_std]
use aidoku::{MangaContentRating, MangaViewer};
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
	MMRCMSSource {
		base_url: "https://onma.me",
		lang: "ar",

		category: "الفئة",
		tags: "العلامات",

		detail_description: "نبذة عن المانجا",
		detail_status_ongoing: "مستمرة",
		detail_status_complete: "مكتملة",

		category_parser: |_, categories| {
			let mut nsfw = MangaContentRating::Safe;
			let mut viewer = MangaViewer::Rtl;
			for category in categories {
				match category.as_str() {
					// "Sexual perversion" | "Mature"
					"انحراف جنسي" | "ناضج" => nsfw = MangaContentRating::Nsfw,
					// Webtoon
					"ويب تون" => viewer = MangaViewer::Scroll,
					_ => continue,
				}
			}
			(nsfw, viewer)
		},
		..Default::default()
	}
}
