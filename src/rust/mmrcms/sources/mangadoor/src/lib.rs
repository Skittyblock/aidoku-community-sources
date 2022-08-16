#![no_std]
use aidoku::{MangaContentRating, MangaViewer};
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
	MMRCMSSource {
		base_url: "http://mangadoor.com",
		lang: "pt-BR",
		category_parser: |_, categories| {
			let mut nsfw = MangaContentRating::Safe;
			let mut viewer = MangaViewer::Rtl;
			for category in categories {
				match category.as_str() {
					"Maduro" | "Hentai" => nsfw = MangaContentRating::Nsfw,
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
		category_mapper: |idx| {
			if idx == 0 {
				String::new()
			} else if (1..=3).contains(&idx) {
				String::from(itoa::Buffer::new().format(idx))
			} else {
				String::from(itoa::Buffer::new().format(idx + 1))
			}
		},
		..Default::default()
	}
}
